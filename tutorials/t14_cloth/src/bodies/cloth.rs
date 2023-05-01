use bevy::{
    prelude::*,
    reflect::TypeUuid,
    render::{
        mesh::{Indices, VertexAttributeValues},
        render_resource::PrimitiveTopology,
    },
};
use bevy_inspector_egui::prelude::*;

use crate::{
    intersect::{ray_sphere_intersect, ray_triangle_intersect},
    math::*,
};

#[derive(Reflect, Default, InspectorOptions, TypeUuid)]
#[uuid = "dfb753d6-6d90-48c7-a304-0b4d57ca9c2f"]
pub struct Cloth {
    num_particles: usize,
    num_triangles: usize,
    indices: Vec<usize>,
    positions: Vec<f32>,
    prev_positions: Vec<f32>,
    rest_positions: Vec<f32>,
    velocities: Vec<f32>,
    inv_mass: Vec<f32>,

    #[inspector(min = 0., max = 100.)]
    bending_compliance: f32,
    #[inspector(min = 0., max = 1.)]
    stretching_compliance: f32,

    bending_ids: Vec<usize>,
    bending_lengths: Vec<f32>,
    stretching_ids: Vec<usize>,
    stretching_lengths: Vec<f32>,

    temp: Vec<f32>,
    grads: Vec<f32>,

    grab_id: Option<usize>,
    grab_inv_mass: f32,

    pub radius: f32, // for raycasting
}

#[derive(Clone, Copy)]
struct Edge {
    id0: usize,
    id1: usize,
    edge_nr: usize,
}
impl Cloth {

    pub fn new(mesh: &Mesh, bending_compliance: f32, offset: &Transform, pin_indices: &[usize]) -> Self {
        let vertices = mesh
            .attribute(Mesh::ATTRIBUTE_POSITION)
            .expect("Cloth mesh requires ATTRIBUTE_POSITION");
        let positions = match vertices {
            VertexAttributeValues::Float32x3(positions) => {
                let x = positions.clone().to_vec();
                x
            }
            _ => panic!("Wrong attribute type"),
        }
        .iter()
        .map(|v| {
            offset
                .transform_point(Vec3::new(v[0], v[1], v[2]))
                .to_array()
        })
        .flatten()
        .collect::<Vec<_>>();

        let indices = match mesh.indices().expect("Cloth mesh requires indices") {
            Indices::U16(v) => v.iter().map(|&i| i as usize).collect::<Vec<_>>(),
            Indices::U32(v) => v.iter().map(|&i| i as usize).collect::<Vec<_>>(),
        };

        let num_particles = positions.len() / 3;
        let num_triangles = indices.len() / 3;
        let mut result = Self {
            num_particles,
            num_triangles,
            indices: indices,
            positions: positions.clone(),
            prev_positions: positions.clone(),
            rest_positions: positions.clone(),
            velocities: vec![0.0; 3 * num_particles],
            inv_mass: vec![0.0; num_particles],
            bending_compliance,
            stretching_compliance: 0.01,
            temp: vec![0.0; 4 * 3],
            grads: vec![0.0; 4 * 3],
            grab_id: None,
            grab_inv_mass: 0.,
            radius: 0.0,

            stretching_ids: vec![],
            stretching_lengths: vec![],
            bending_ids: vec![],
            bending_lengths: vec![],
        };

        let neighors = result.find_tri_neighbors();
        let num_tris = result.indices.len() / 3;
        let mut edge_ids = vec![];
        let mut tri_pair_ids = vec![];

        for i in 0..num_tris {
            for j in 0..3 {
                let id0 = result.indices[3 * i + j];
                let id1 = result.indices[3 * i + (j + 1) % 3];

                // each edge only once
                let n = neighors[3 * i + j];
                if n < 0 || id0 < id1 {
                    edge_ids.push(id0);
                    edge_ids.push(id1);
                }
                // tri pair
                if n >= 0 {
                    // opposite ids
                    let ni = (n / 3) as usize;
                    let nj = (n % 3) as usize;
                    let id2 = result.indices[3 * i + (j + 2) % 3];
                    let id3 = result.indices[3 * ni + (nj + 2) % 3];
                    tri_pair_ids.push(id0);
                    tri_pair_ids.push(id1);
                    tri_pair_ids.push(id2);
                    tri_pair_ids.push(id3);
                }
            }
        }

        result.stretching_ids = edge_ids;
        result.bending_ids = tri_pair_ids;
        result.stretching_lengths = vec![0.0; result.stretching_ids.len() / 2];
        result.init_physics();

        result.pin_indices(pin_indices);
        // 
        result
    }

    pub fn tri_count(&self) -> usize {
        self.num_triangles
    }
    pub fn vert_count(&self) -> usize {
        self.num_particles
    }

    pub fn init_physics(&mut self) {
        let num_tris = self.indices.len() / 3;
        let mut e0 = [0f32; 3];
        let mut e1 = [0f32; 3];
        let mut c = [0f32; 3];

        for i in 0..num_tris {
            let id0 = self.indices[3 * i];
            let id1 = self.indices[3 * i + 1];
            let id2 = self.indices[3 * i + 2];
            vecSetDiff(&mut e0, 0, &self.positions, id1, &self.positions, id0, 1.0);
            vecSetDiff(&mut e1, 0, &self.positions, id2, &self.positions, id0, 1.0);
            vecSetCross(&mut c, 0, &e0, 0, &e1, 0);
            let a = 0.5 * vecLengthSquared(&c, 0).sqrt();
            let p_inv_mass = if a > 0.0 { 1.0 / (a * 3.0) } else { 0.0 };
            self.inv_mass[id0] += p_inv_mass;
            self.inv_mass[id1] += p_inv_mass;
            self.inv_mass[id2] += p_inv_mass;
        }

        for i in 0..self.stretching_lengths.len() {
            let id0 = self.stretching_ids[2 * i];
            let id1 = self.stretching_ids[2 * i + 1];
            self.stretching_lengths[i] =
                vecDistSquared(&self.positions, id0, &self.positions, id1).sqrt();
        }

        for i in 0..self.bending_lengths.len() {
            let id0 = self.bending_ids[4 * i + 2];
            let id1 = self.bending_ids[4 * i + 3];
            self.bending_lengths[i] =
                vecDistSquared(&self.positions, id0, &self.positions, id1).sqrt();
        }
    }

    pub fn pin_indices(&mut self, indices: &[usize]) {
        for i in 0..indices.len() {            
            self.inv_mass[indices[i]] = 0.0;                            
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
        self.solve_stretching(dt);
        self.solve_bending(dt);
    }

    pub fn post_solve(&mut self, dt: f32) {
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

    fn solve_stretching(&mut self, dt: f32) {
        let alpha = self.stretching_compliance / dt / dt;

        for i in 0..self.stretching_lengths.len() {
            let id0 = self.stretching_ids[2 * i];
            let id1 = self.stretching_ids[2 * i + 1];
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
            let rest_len = self.stretching_lengths[i];
            let c = len - rest_len;
            let s = -c / (w + alpha);
            vecAdd(&mut self.positions, id0, &self.grads, 0, s * w0);
            vecAdd(&mut self.positions, id1, &self.grads, 0, -s * w1);
        }
    }

    fn solve_bending(&mut self, dt: f32) {
        let alpha = self.bending_compliance / dt / dt;

        for i in 0..self.bending_lengths.len() {
            let id0 = self.bending_ids[4 * i + 2];
            let id1 = self.bending_ids[4 * i + 3];
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
            let rest_len = self.bending_lengths[i];
            let c = len - rest_len;
            let s = -c / (w + alpha);
            vecAdd(&mut self.positions, id0, &self.grads, 0, s * w0);
            vecAdd(&mut self.positions, id1, &self.grads, 0, -s * w1);
        }
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

    pub fn update_visual_mesh(&mut self, trans: &Transform, mesh: &mut Mesh) {
        let indices = self.indices.iter().map(|i| *i as u32).collect::<Vec<u32>>();
        let positions = self
            .positions
            .chunks_exact(3)
            .map(|v| {
                [
                    v[0] - trans.translation.x,
                    v[1] - trans.translation.y,
                    v[2] - trans.translation.z,
                ]
            })
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
            for i in 0..self.indices.len() / 3 {
                let index = i * 3;
                let p0 = Vec3::new(
                    self.positions[self.indices[index] * 3],
                    self.positions[self.indices[index] * 3 + 1],
                    self.positions[self.indices[index] * 3 + 2],
                );
                let p1 = Vec3::new(
                    self.positions[self.indices[index + 1] * 3],
                    self.positions[self.indices[index + 1] * 3 + 1],
                    self.positions[self.indices[index + 1] * 3 + 2],
                );
                let p2 = Vec3::new(
                    self.positions[self.indices[index + 2] * 3],
                    self.positions[self.indices[index + 2] * 3 + 1],
                    self.positions[self.indices[index + 2] * 3 + 2],
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
        let indices = self.indices.iter().map(|i| *i as u32).collect::<Vec<u32>>();
        let positions = self
            .positions
            .chunks_exact(3)
            .map(|v| [v[0], v[1], v[2]])
            .collect::<Vec<[f32; 3]>>();

        mesh.set_indices(Some(Indices::U32(indices)));
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.duplicate_vertices();
        mesh.compute_flat_normals();
    }

    fn find_tri_neighbors(&self) -> Vec<usize> {
        // create common edges
        let mut edges: Vec<Edge> = vec![];

        for i in 0..self.num_triangles {
            for j in 0..3 {
                let id0 = self.indices[3 * i + j];
                let id1 = self.indices[3 * i + (j + 1) % 3];
                edges.push(Edge {
                    id0: id0.min(id1),
                    id1: id0.max(id1),
                    edge_nr: 3 * i + j,
                });
            }
        }

        // sort so common edges are next to each other

        edges.sort_by(|a, b| -> core::cmp::Ordering {
            return if (a.id0 < b.id0) || (a.id0 == b.id0 && a.id1 < b.id1) {
                core::cmp::Ordering::Less
            } else {
                core::cmp::Ordering::Greater
            };
        });
        // find matchign edges

        let mut neighbors = vec![0usize; 3 * self.num_triangles];
        //NOTE:  Was -1 for all as open edges are -1, but we want to be able to index into the array

        let mut nr = 0;

        while nr < edges.len() {
            let e0 = edges[nr];
            nr += 1;
            if nr < edges.len() {
                let e1 = edges[nr];
                if e0.id0 == e1.id0 && e0.id1 == e1.id1 {
                    neighbors[e0.edge_nr] = e1.edge_nr;
                    neighbors[e1.edge_nr] = e0.edge_nr;
                }
                nr += 1;
            }
        }
        return neighbors;
    }
}

impl From<&Cloth> for Mesh {
    fn from(sb: &Cloth) -> Self {
        // generate mesh
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        let indices = sb.indices.iter().map(|i| *i as u32).collect::<Vec<u32>>();
        let positions = sb
            .positions
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
