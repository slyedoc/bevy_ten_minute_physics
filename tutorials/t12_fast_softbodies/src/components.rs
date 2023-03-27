use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use bevy_inspector_egui::prelude::*;

use crate::{
    intersect::{ray_sphere_intersect, ray_triangle_intersect},
    models::TetMesh,
};

#[derive(Reflect, Component)]
#[reflect(Component)]
pub struct Ball(pub f32);

impl Default for Ball {
    fn default() -> Self {
        Self(0.5)
    }
}

#[derive(Reflect, Component, Default, Deref, DerefMut)]
#[reflect(Component)]
pub struct Velocity(pub Vec3);

#[derive(Reflect, Component, Default, InspectorOptions)]
#[reflect(Component, InspectorOptions)]
pub struct SoftBody {
    indices: Vec<usize>,
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
    pub fn new(tet_mesh: &TetMesh, edge_compliance: f32, volume_compliance: f32) -> Self {
        let num_particles = tet_mesh.tet_vertices.len() / 3;
        let num_tets = tet_mesh.tet_indices.len() / 4;

        let mut result = Self {
            indices: tet_mesh.indices.clone(),
            num_particles,
            num_tets,
            positions: tet_mesh.vertices.clone(), // -
            prev_positions: tet_mesh.tet_vertices.clone(),
            velocities: vec![0.0; 3 * num_particles],
            tet_ids: tet_mesh.tet_indices.clone(),
            edge_ids: tet_mesh.tet_edge_ids.clone(),
            rest_volumn: vec![0.0; num_tets],
            edge_lengths: vec![0.0; tet_mesh.tet_edge_ids.len() / 2],
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
        result
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
        // self.updateMeshes();
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

    // Returns a mesh and the center of the mesh and the radius of the mesh
    pub fn update_info(&mut self) -> (Mesh, Vec3) {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        let indices = self.indices.iter().map(|i| *i as u32).collect::<Vec<u32>>();
        let mut positions = self
            .positions
            .chunks_exact(3)
            .map(|v| [v[0], v[1], v[2]])
            .collect::<Vec<[f32; 3]>>();

        // find avg position and radius of the mesh
        let mut avg_pos = [0.0, 0.0, 0.0];

        for pos in &positions {
            avg_pos[0] += pos[0];
            avg_pos[1] += pos[1];
            avg_pos[2] += pos[2];
        }
        avg_pos[0] /= positions.len() as f32;
        avg_pos[1] /= positions.len() as f32;
        avg_pos[2] /= positions.len() as f32;

        // find max distance from avg position
        let mut max_dist = 0.0;
        for pos in &positions {
            let dist = (pos[0] - avg_pos[0]).powi(2)
                + (pos[1] - avg_pos[1]).powi(2)
                + (pos[2] - avg_pos[2]).powi(2);
            if dist > max_dist {
                max_dist = dist;
            }
        }
        self.radius = max_dist.sqrt();

        // update positions to be relative to the center of the mesh
        for pos in &mut positions {
            pos[0] -= avg_pos[0];
            pos[1] -= avg_pos[1];
            pos[2] -= avg_pos[2];
        }

        // generate mesh
        mesh.set_indices(Some(Indices::U32(indices)));
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, positions);
        mesh.duplicate_vertices();
        mesh.compute_flat_normals();

        (mesh, Vec3::new(avg_pos[0], avg_pos[1], avg_pos[2]))
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
}

#[allow(dead_code, non_snake_case)]
fn vecSetZero(a: &mut Vec<f32>, anr: usize) {
    let ar = anr * 3;
    a[ar] = 0.0;
    a[ar + 1] = 0.0;
    a[ar + 2] = 0.0;
}

#[allow(non_snake_case)]
fn vecScale(a: &mut Vec<f32>, anr: usize, scale: f32) {
    let ar = anr * 3;
    a[ar] *= scale;
    a[ar + 1] *= scale;
    a[ar + 2] *= scale;
}

#[allow(non_snake_case)]
fn vecCopy(a: &mut [f32], anr: usize, b: &[f32], bnr: usize) {
    let ar = anr * 3;
    let br = bnr * 3;
    a[ar] = b[br];
    a[ar + 1] = b[br + 1];
    a[ar + 2] = b[br + 2];
}

#[allow(non_snake_case)]
fn vecAdd(a: &mut [f32], anr: usize, b: &[f32], bnr: usize, scale: f32) {
    let ar = anr * 3;
    let br = bnr * 3;
    a[ar] += b[br] * scale;
    a[ar + 1] += b[br + 1] * scale;
    a[ar + 2] += b[br + 2] * scale;
}

#[allow(non_snake_case)]
fn vecSetDiff(
    dst: &mut [f32],
    dnr: usize,
    a: &[f32],
    anr: usize,
    b: &[f32],
    bnr: usize,
    scale: f32,
) {
    let dr = dnr * 3;
    let ar = anr * 3;
    let br = bnr * 3;
    dst[dr] = (a[ar] - b[br]) * scale;
    dst[dr + 1] = (a[ar + 1] - b[br + 1]) * scale;
    dst[dr + 2] = (a[ar + 2] - b[br + 2]) * scale;
}

#[allow(non_snake_case)]
fn vecLengthSquared(a: &[f32], anr: usize) -> f32 {
    let ar = anr * 3;
    let a0 = a[ar];
    let a1 = a[ar + 1];
    let a2 = a[ar + 2];
    return a0 * a0 + a1 * a1 + a2 * a2;
}

#[allow(non_snake_case)]
fn vecDistSquared(a: &[f32], anr: usize, b: &[f32], bnr: usize) -> f32 {
    let ar = anr * 3;
    let br = bnr * 3;
    let a0 = a[ar] - b[br];
    let a1 = a[ar + 1] - b[br + 1];
    let a2 = a[ar + 2] - b[br + 2];
    return a0 * a0 + a1 * a1 + a2 * a2;
}

#[allow(non_snake_case)]
fn vecDot(a: &[f32], anr: usize, b: &[f32], bnr: usize) -> f32 {
    let ar = anr * 3;
    let br = bnr * 3;
    return a[ar] * b[br] + a[ar + 1] * b[br + 1] + a[ar + 2] * b[br + 2];
}

#[allow(non_snake_case)]
fn vecSetCross(a: &mut [f32], anr: usize, b: &[f32], bnr: usize, c: &[f32], cnr: usize) {
    let ar = anr * 3;
    let br = bnr * 3;
    let cr = cnr * 3;
    a[ar] = b[br + 1] * c[cr + 2] - b[br + 2] * c[cr + 1];
    a[ar + 1] = b[br + 2] * c[cr + 0] - b[br + 0] * c[cr + 2];
    a[ar + 2] = b[br + 0] * c[cr + 1] - b[br + 1] * c[cr + 0];
}
