use bevy::{
    prelude::*,
    render::{mesh::{Indices, VertexAttributeValues}, render_resource::PrimitiveTopology}, reflect::TypeUuid,
};
use bevy_inspector_egui::prelude::*;

use crate::{
    intersect::{ray_sphere_intersect, ray_triangle_intersect},
    assets::TetMesh, spatial_hash::SpatialHash,
    math::*,
};


#[derive(Reflect, Default, InspectorOptions, TypeUuid)]
#[uuid = "bbf321bb-1e8e-4b03-88d6-152b7f10e9db"]
pub struct SoftBody {
    // visual mesh
    #[inspector()]
    visual_indices: Vec<usize>,
    visual_vertices: Vec<f32>,
    num_vis_verts: usize,
    skinning_info: Vec<f32>,

    // tet mesh
    num_particles: usize,
    num_tets: usize,
    positions: Vec<f32>,
    prev_positions: Vec<f32>,
    velocities: Vec<f32>,
    tet_ids: Vec<usize>,
    edge_ids: Vec<usize>,
    rest_volumn: Vec<f32>,
    edge_lengths: Vec<f32>,
    inv_mass: Vec<f32>,
    #[inspector(min = 0., max = 100.)]
    edge_compliance: f32,
    #[inspector(min = 0., max = 1.)]
    volume_compliance: f32,

    temp: Vec<f32>,
    grads: Vec<f32>,
    grab_id: Option<usize>,
    grab_inv_mass: f32,

    pub radius: f32, // for raycasting
}

const VOLUME_ID_ORDER: [[usize; 3]; 4] = [[1, 3, 2], [0, 2, 3], [0, 3, 1], [0, 1, 2]];

impl SoftBody {
    pub fn new(mesh: &TetMesh, edge_compliance: f32, volume_compliance: f32) -> Self {
        let num_particles = mesh.tet_vertices.len() / 3;
        let num_tets = mesh.tet_indices.len() / 4;
        let num_vis_verts = mesh.vertices.len() / 3;

        let mut result = Self {
            visual_indices: mesh.indices.clone(),
            visual_vertices: mesh.vertices.clone(),
            num_vis_verts: num_vis_verts,
            skinning_info: vec![0.0; 4 * num_vis_verts],
            
            // tet mesh
            num_particles,
            num_tets,
            positions: mesh.tet_vertices.clone(),
            prev_positions: mesh.tet_vertices.clone(),
            velocities: vec![0.0; 3 * num_particles],
            tet_ids: mesh.tet_indices.clone(),
            edge_ids: mesh.tet_edge_ids.clone(),
            rest_volumn: vec![0.0; num_tets],
            edge_lengths: vec![0.0; mesh.tet_edge_ids.len() / 2],
            inv_mass: vec![0.0; num_particles],
            edge_compliance,
            volume_compliance,
            temp: vec![0.0; 4 * 3],
            grads: vec![0.0; 4 * 3],
            grab_id: None,
            grab_inv_mass: 0.,
            radius: 0.0,
        };

        result.init_physics();

        result.computeSkinningInfo();
        result
    }

    #[allow(non_snake_case)]
    pub fn computeSkinningInfo(&mut self)
    {
        // create a hash for all vertices of the visual mesh
        let mut hash = SpatialHash::new(0.05, self.num_vis_verts);
        hash.create(&self.visual_vertices.chunks(3)
        .map(|v| Vec3::new(v[0], v[1], v[2]))
        .collect::<Vec<Vec3>>());        

        self.skinning_info.fill(-1.0);		// undefined

        let mut min_dist = vec![ f32::MAX; self.num_vis_verts ];
        
        let border = 0.05;

        // each tet searches for containing vertices
        let mut tet_center = [0.0; 3];
        let mut mat = [0.0; 9];
        let mut bary = [0.0; 4];

        for i in 0..self.num_tets {

            // compute bounding sphere of tet
            tet_center.fill(0.0);
            for j in 0..4 {
                vecAdd(&mut tet_center, 0, &self.positions, self.tet_ids[4 * i + j], 0.25);
            }

            let mut r_max = 0.0f32;
            for j in 0..4 {
                let r2 = vecDistSquared(&tet_center, 0, &self.positions, self.tet_ids[4 * i + j]);
                r_max = r_max.max(r2.sqrt());
            }

            r_max += border;

            hash.query(&tet_center, 0, r_max);
            if hash.query_ids.len() == 0 {
                continue;
            }

            let id0 = self.tet_ids[4 * i];
            let id1 = self.tet_ids[4 * i + 1];
            let id2 = self.tet_ids[4 * i + 2];
            let id3 = self.tet_ids[4 * i + 3];

            vecSetDiff(&mut mat, 0, &self.positions, id0, &self.positions, id3, 1.0);
            vecSetDiff(&mut mat, 1, &self.positions, id1, &self.positions, id3, 1.0);
            vecSetDiff(&mut mat, 2, &self.positions, id2, &self.positions, id3, 1.0);

            matSetInverse(&mut mat);

            for j in 0..hash.query_ids.len() {
                let id = hash.query_ids[j];

                // we already have skinning info
                if min_dist[id] <= 0.0 {
                    continue;
                }

                if vecDistSquared(&self.visual_vertices, id, &tet_center, 0) > r_max * r_max {
                    continue;
                }

                // compute barycentric coords for candidate
                vecSetDiff(&mut bary, 0, &self.visual_vertices, id, &self.positions, id3, 1.0);
                matSetMult(&mat, &mut bary, 0);
                bary[3] = 1.0 - bary[0] - bary[1] - bary[2];

                let mut dist = 0.0f32;
                for k in 0..4 {
                    dist = dist.max( -bary[k]);
                }
                    
                if dist < min_dist[id] {
                    min_dist[id] = dist;
                    self.skinning_info[4 * id] = i as f32;
                    self.skinning_info[4 * id + 1] = bary[0];
                    self.skinning_info[4 * id + 2] = bary[1];
                    self.skinning_info[4 * id + 3] = bary[2];
                }
            }
        }
    }

  
    pub fn get_tet_volume(&mut self, nr: usize) -> f32 {
        let id0 = self.tet_ids[4 * nr];
        let id1 = self.tet_ids[4 * nr + 1];
        let id2 = self.tet_ids[4 * nr + 2];
        let id3 = self.tet_ids[4 * nr + 3];
        vecSetDiff(
            &mut self.temp,
            0,
            &self.positions,
            id1,
            &self.positions,
            id0,
            1.0,
        );
        vecSetDiff(
            &mut self.temp,
            1,
            &self.positions,
            id2,
            &self.positions,
            id0,
            1.0,
        );
        vecSetDiff(
            &mut self.temp,
            2,
            &self.positions,
            id3,
            &self.positions,
            id0,
            1.0,
        );

        // Cant borrow self.temp as mutable and immutable, doing it manually
        // vecSetCross(&mut self.temp, 3, &self.temp, 0, &self.temp, 1);
        let ar = 3 * 3;
        let br = 0 * 3;
        let cr = 1 * 3;
        self.temp[ar] =
            self.temp[br + 1] * self.temp[cr + 2] - self.temp[br + 2] * self.temp[cr + 1];
        self.temp[ar + 1] =
            self.temp[br + 2] * self.temp[cr + 0] - self.temp[br + 0] * self.temp[cr + 2];
        self.temp[ar + 2] =
            self.temp[br + 0] * self.temp[cr + 1] - self.temp[br + 1] * self.temp[cr + 0];

        return vecDot(&self.temp, 3, &self.temp, 2) / 6.0;
    }

    pub fn init_physics(&mut self) {
        for i in 0..self.num_tets {
            let vol = self.get_tet_volume(i);
            self.rest_volumn[i] = vol;
            let p_inv_mass = if vol > 0.0 { 1.0 / (vol / 4.0) } else { 0.0 };
            self.inv_mass[self.tet_ids[4 * i]] += p_inv_mass;
            self.inv_mass[self.tet_ids[4 * i + 1]] += p_inv_mass;
            self.inv_mass[self.tet_ids[4 * i + 2]] += p_inv_mass;
            self.inv_mass[self.tet_ids[4 * i + 3]] += p_inv_mass;
        }
        for i in 0..self.edge_lengths.len() {
            let id0 = self.edge_ids[2 * i];
            let id1 = self.edge_ids[2 * i + 1];
            self.edge_lengths[i] =
                vecDistSquared(&self.positions, id0, &self.positions, id1).sqrt();
        }
    }

    pub fn pre_solve(&mut self, dt: f32, gravity: Vec3) {
        for i in 0..self.num_particles {
            if self.inv_mass[i] == 0.0 {
                continue;
            }
            //vecAdd(this.vel,i, gravity,0, dt);
            self.velocities[i * 3] += gravity[0] * dt;
            self.velocities[i * 3 + 1] += gravity[1] * dt;
            self.velocities[i * 3 + 2] += gravity[2] * dt;

            vecCopy(&mut self.prev_positions, i, &self.positions, i);
            vecAdd(&mut self.positions, i, &self.velocities, i, dt);
            let y = self.positions[3 * i + 1];
            if y < 0.0 {
                vecCopy(&mut self.positions, i, &self.prev_positions, i);
                self.positions[3 * i + 1] = 0.0;
            }
        }
    }

    pub fn solve(&mut self, dt: f32) {
        self.solve_edges(self.edge_compliance, dt);
        self.solve_volumes(self.volume_compliance, dt);
    }

    pub fn post_solve( &mut self, dt: f32) {
        

        for i in 0..self.num_particles {
            if self.inv_mass[i] == 0.0 {
                continue;
            }
            vecSetDiff(
                &mut self.velocities,
                i,
                &self.positions,
                i,
                &self.prev_positions,
                i,
                1.0 / dt,
            );
        }
    }

    fn solve_edges(&mut self, compliance: f32, dt: f32) {
        let alpha = compliance / dt / dt;

        for i in 0..self.edge_lengths.len() {
            let id0 = self.edge_ids[2 * i];
            let id1 = self.edge_ids[2 * i + 1];
            let w0 = self.inv_mass[id0];
            let w1 = self.inv_mass[id1];
            let w = w0 + w1;
            if w == 0.0 {
                continue;
            }

            vecSetDiff(
                &mut self.grads,
                0,
                &self.positions,
                id0,
                &self.positions,
                id1,
                1.0,
            );
            let len = vecLengthSquared(&self.grads, 0).sqrt();
            if len == 0.0 {
                continue;
            }
            vecScale(&mut self.grads, 0, 1.0 / len);
            let rest_len = self.edge_lengths[i];
            let c = len - rest_len;
            let s = -c / (w + alpha);
            vecAdd(&mut self.positions, id0, &self.grads, 0, s * w0);
            vecAdd(&mut self.positions, id1, &self.grads, 0, -s * w1);
        }
    }

    fn solve_volumes(&mut self, compliance: f32, dt: f32) {
        let alpha = compliance / dt / dt;

        for i in 0..self.num_tets {
            let mut w = 0.0;

            for j in 0..4 {
                let id0 = self.tet_ids[4 * i + VOLUME_ID_ORDER[j][0]];
                let id1 = self.tet_ids[4 * i + VOLUME_ID_ORDER[j][1]];
                let id2 = self.tet_ids[4 * i + VOLUME_ID_ORDER[j][2]];

                vecSetDiff(
                    &mut self.temp,
                    0,
                    &self.positions,
                    id1,
                    &self.positions,
                    id0,
                    1.0,
                );
                vecSetDiff(
                    &mut self.temp,
                    1,
                    &self.positions,
                    id2,
                    &self.positions,
                    id0,
                    1.0,
                );
                vecSetCross(&mut self.grads, j, &self.temp, 0, &self.temp, 1);
                vecScale(&mut self.grads, j, 1.0 / 6.0);

                w += self.inv_mass[self.tet_ids[4 * i + j]] * vecLengthSquared(&self.grads, j);
            }
            if w == 0.0 {
                continue;
            }

            let vol = self.get_tet_volume(i);
            let rest_vol = self.rest_volumn[i];
            let c = vol - rest_vol;
            let s = -c / (w + alpha);

            for j in 0..4 {
                let id = self.tet_ids[4 * i + j];
                vecAdd(
                    &mut self.positions,
                    id,
                    &self.grads,
                    j,
                    s * self.inv_mass[id],
                )
            }
        }
    }
    
    pub fn create_tet_mesh(&self) -> Mesh {
        let mut mesh = Mesh::new(PrimitiveTopology::LineList);
        let indices = self.edge_ids.iter().map(|i| *i as u32).collect::<Vec<u32>>();
        let positions = self
            .positions
            .chunks_exact(3)
            .map(|v| [v[0], v[1], v[2]])
            .collect::<Vec<[f32; 3]>>();

        mesh.set_indices(Some(Indices::U32(indices)));
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh    
    }
    

    // moves position changes to local space and updates transform position, call before update meshes
    pub fn update_transform(&mut self, trans: &mut Transform) {
        
        // find avg position and radius of the mesh
        let mut avg_pos = Vec3::ZERO;     
        let len = self.positions.len() / 3;
        for pos in self.positions.chunks_exact(3) {
             avg_pos[0] += pos[0];
             avg_pos[1] += pos[1];
             avg_pos[2] += pos[2];
        }
        avg_pos[0] /= len as f32;
        avg_pos[1] /= len as f32;
        avg_pos[2] /= len as f32;

        // find max distance from avg position
        let mut max_dist = 0.0;
        for pos in self.positions.chunks_exact(3) {
            
            let dist = (pos[0] - avg_pos[0]).powi(2)
                 + (pos[1] - avg_pos[1]).powi(2)
                 + (pos[2] - avg_pos[2]).powi(2);
             if dist > max_dist {
                 max_dist = dist;
             }
        }
        self.radius = max_dist.sqrt();
        trans.translation = avg_pos;

    // update positions to be relative to the center of the mesh
    //     for pos in self.positions.chunks_exact_mut(3) {
    //          pos[0] -= avg_pos[0];
    //          pos[1] -= avg_pos[1];
    //          pos[2] -= avg_pos[2];
    //     }

    //     for pos in self.prev_positions.chunks_exact_mut(3) {
    //         pos[0] -= avg_pos[0];
    //         pos[1] -= avg_pos[1];
    //         pos[2] -= avg_pos[2];
    //    }

    //     for pos in self.visual_vertices.chunks_exact_mut(3) {
    //         pos[0] -= avg_pos[0];
    //         pos[1] -= avg_pos[1];
    //         pos[2] -= avg_pos[2];
    //     }

    }

    pub fn update_tet_mesh(&mut self, trans: &Transform, mesh: &mut Mesh) {
        
        let positions = mesh.attribute_mut(Mesh::ATTRIBUTE_POSITION).unwrap();
        for i in 0..(self.positions.len() / 3) {
            match positions {
                VertexAttributeValues::Float32x3(positions) => {
                    positions[i][0] = self.positions[i * 3] - trans.translation.x;
                    positions[i][1] = self.positions[i * 3 + 1] - trans.translation.y;
                    positions[i][2] = self.positions[i * 3 + 2] - trans.translation.z;
                }
                _ => panic!("Wrong attribute type"),
            }            
        }        
    }

    pub fn update_visual_mesh(&mut self, trans: &Transform, mesh: &mut Mesh) {
        let mut nr = 0;
        for i in 0..self.num_vis_verts {
            let mut tet_nr = (self.skinning_info[nr] * 4.0) as usize; // TODO: check this
            nr += 1;
            // dont think this can happen
            // if tet_nr < 0 { 
            //     // dont think this can happen
            //     nr += 3;
            //     continue;
            // }
            let b0 = self.skinning_info[nr];
            nr += 1;
            let b1 = self.skinning_info[nr];
            nr += 1;
            let b2 = self.skinning_info[nr];
            nr += 1;
            let b3 = 1.0 - b0 - b1 - b2;
            let id0 = self.tet_ids[tet_nr];
            tet_nr += 1;
            let id1 = self.tet_ids[tet_nr];
            tet_nr += 1;
            let id2 = self.tet_ids[tet_nr];
            tet_nr += 1;
            let id3 = self.tet_ids[tet_nr];
            //tet_nr += 1;
            vecSetZero(&mut self.visual_vertices, i);
            vecAdd(&mut self.visual_vertices, i, &self.positions, id0, b0);
            vecAdd(&mut self.visual_vertices, i, &self.positions, id1, b1);
            vecAdd(&mut self.visual_vertices, i, &self.positions, id2, b2);
            vecAdd(&mut self.visual_vertices, i, &self.positions, id3, b3);
        }


        let indices = self.visual_indices.iter().map(|i| *i as u32).collect::<Vec<u32>>();
        let positions = self
            .visual_vertices
            .chunks_exact(3)
            .map(|v| [v[0] - trans.translation.x, v[1] - trans.translation.y, v[2] - trans.translation.z])
            .collect::<Vec<[f32; 3]>>();

        
        mesh.set_indices(Some(Indices::U32(indices)));
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.duplicate_vertices();
        mesh.compute_flat_normals();
    }

    // returns distance to closest point
    pub fn intersect(&mut self, ray: Ray, trans: &Transform) -> Option<f32> {
        
        if let Some((_, _)) = ray_sphere_intersect(ray, trans.translation, self.radius) {
            // ray intersects sphere, now check if it intersects the mesh
            let mut min_dist = f32::MAX;
            for i in 0..self.tet_ids.len() / 3 {
                let index = i * 3;
                let p0 = Vec3::new(
                    self.positions[self.tet_ids[index] * 3],
                    self.positions[self.tet_ids[index] * 3 + 1],
                    self.positions[self.tet_ids[index] * 3 + 2],
                );
                let p1 = Vec3::new(
                    self.positions[self.tet_ids[index + 1] * 3],
                    self.positions[self.tet_ids[index + 1] * 3 + 1],
                    self.positions[self.tet_ids[index + 1] * 3 + 2],
                );
                let p2 = Vec3::new(
                    self.positions[self.tet_ids[index + 2] * 3],
                    self.positions[self.tet_ids[index + 2] * 3 + 1],
                    self.positions[self.tet_ids[index + 2] * 3 + 2],
                );
                if let Some(dist) = ray_triangle_intersect(ray, p0, p1, p2) {
                    if dist < min_dist {
                        min_dist = dist;
                    }
                }
            }

            if min_dist < f32::MAX {
                return Some(min_dist);
            }
        }
        None
    }

    pub fn start_grab(&mut self, pos: Vec3) {
        let mut p = [pos.x, pos.y, pos.z];
        let mut min_d2 = f32::MAX;
        self.grab_id = None;
        for i in 0..self.num_particles {
            let d2 = vecDistSquared(&mut p, 0, &self.positions, i);
            if d2 < min_d2 {
                min_d2 = d2;
                self.grab_id = Some(i);
            }
        }

        if let Some(index) = self.grab_id {
            self.grab_inv_mass = self.inv_mass[index];
            self.inv_mass[index] = 0.0;
            vecCopy(&mut self.positions, index, &p, 0);
        }
    }

    pub fn move_grabbed(&mut self, pos: Vec3, _vel: Vec3) {
        if let Some(index) = self.grab_id {
            let p = [pos.x, pos.y, pos.z];
            vecCopy(&mut self.positions, index, &p, 0);
        }
    }

    pub fn end_grab(&mut self, _pos: Vec3, vel: Vec3) {
        if let Some(index) = self.grab_id {
            self.inv_mass[index] = self.grab_inv_mass;
            let v = [vel.x, vel.y, vel.z];
            vecCopy(&mut self.velocities, index, &v, 0);
        }
        self.grab_id = None;
    }

    pub fn update_mesh(&mut self, mesh: &mut Mesh) {
        let indices = self.visual_indices.iter().map(|i| *i as u32).collect::<Vec<u32>>();
        let positions = self
            .visual_vertices
            .chunks_exact(3)
            .map(|v| [v[0], v[1], v[2]])
            .collect::<Vec<[f32; 3]>>();

        mesh.set_indices(Some(Indices::U32(indices)));
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.duplicate_vertices();
        mesh.compute_flat_normals();
    }
}

impl From<&SoftBody> for Mesh {
    fn from(sb: &SoftBody) -> Self {
        // let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        // var geometry = new THREE.BufferGeometry();
        // geometry.setAttribute('position', new THREE.BufferAttribute(this.pos, 3));
        // geometry.setIndex(tetMesh.edgeIds);
        // var lineMaterial = new THREE.LineBasicMaterial({color: 0xffffff, linewidth: 2});
        // this.tetMesh = new THREE.LineSegments(geometry, lineMaterial);
        // this.tetMesh.visible = true;
        // scene.add(this.tetMesh);
        // this.tetMesh.visible = false;

        // generate mesh
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        let indices = sb.visual_indices.iter().map(|i| *i as u32).collect::<Vec<u32>>();
        let positions = sb
            .visual_vertices
            .chunks_exact(3)
            .map(|v| [v[0], v[1], v[2]])
            .collect::<Vec<[f32; 3]>>();

        mesh.set_indices(Some(Indices::U32(indices)));
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.duplicate_vertices();
        mesh.compute_flat_normals();
        mesh    
    }
}

