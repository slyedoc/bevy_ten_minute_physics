use bevy::{
    prelude::*,
    render::{mesh::Indices, render_resource::PrimitiveTopology},
};
use bevy_inspector_egui::prelude::*;

use crate::models::TetMesh;

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
}

const VOLUME_ID_ORDER: [[usize; 3]; 4] = [[1, 3, 2], [0, 2, 3], [0, 3, 1], [0, 1, 2]];

impl SoftBody {
    pub fn new(tet_mesh: TetMesh, edge_compliance: f32, volume_compliance: f32) -> Self {
        let num_particles = tet_mesh.vertices.len() / 3;
        let num_tets = tet_mesh.tetIds.len() / 4;

        let mut result = Self {
            indices: tet_mesh.indices,
            num_particles,
            num_tets,
            positions: tet_mesh.vertices.clone(),
            prev_positions: tet_mesh.vertices.clone(),
            velocities: vec![0.0; 3 * num_particles],
            tet_ids: tet_mesh.tetIds.clone(),
            edge_ids: tet_mesh.tetEdgeIds.clone(),
            rest_volumn: vec![0.0; num_tets],
            edge_lengths: vec![0.0; tet_mesh.tetEdgeIds.len() / 2],
            inv_mass: vec![0.0; num_particles],
            edge_compliance,
            volume_compliance,
            temp: vec![0.0; 4 * 3],
            grads: vec![0.0; 4 * 3],
            grab_id: None,
            grab_inv_mass: 0.,
        };

        result.init_physics();
        result
    }

    pub fn getTetVolume(&mut self, nr: usize) -> f32 {
        let id0 = self.tet_ids[4 * nr];
        let id1 = self.tet_ids[4 * nr + 1];
        let id2 = self.tet_ids[4 * nr + 2];
        let id3 = self.tet_ids[4 * nr + 3];
        vecSetDiff(&mut self.temp, 0, &self.positions, id1, &self.positions, id0, 1.0);
        vecSetDiff(&mut self.temp, 1, &self.positions, id2, &self.positions, id0, 1.0);
        vecSetDiff(&mut self.temp, 2, &self.positions, id3, &self.positions, id0, 1.0);
        
        // Cant borrow self.temp as mutable and immutable, doing it manually
        // vecSetCross(&mut self.temp, 3, &self.temp, 0, &self.temp, 1);
        let ar = 3 * 3;
        let br = 0 * 3;
        let cr = 1 * 3;
        self.temp[ar] = self.temp[br + 1] * self.temp[cr + 2] - self.temp[br + 2] * self.temp[cr + 1];
        self.temp[ar + 1] = self.temp[br + 2] * self.temp[cr + 0] - self.temp[br + 0] * self.temp[cr + 2];
        self.temp[ar + 2] = self.temp[br + 0] * self.temp[cr + 1] - self.temp[br + 1] * self.temp[cr + 0];
        
        return vecDot(&self.temp, 3, &self.temp, 2) / 6.0;
    }

    pub fn init_physics(&mut self) {
        for i in 0..self.num_tets {
            let vol = self.getTetVolume(i);
            self.rest_volumn[i] = vol;
            let pInvMass = if vol > 0.0 { 1.0 / (vol / 4.0) } else { 0.0 };
            self.inv_mass[self.tet_ids[4 * i]] += pInvMass;
            self.inv_mass[self.tet_ids[4 * i + 1]] += pInvMass;
            self.inv_mass[self.tet_ids[4 * i + 2]] += pInvMass;
            self.inv_mass[self.tet_ids[4 * i + 3]] += pInvMass;
        }
        for i in 0..self.edge_lengths.len() {
            let id0 = self.edge_ids[2 * i];
            let id1 = self.edge_ids[2 * i + 1];
            self.edge_lengths[i] = vecDistSquared(&self.positions, id0, &self.positions, id1).sqrt();
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
            
            vecCopy(&mut self.prev_positions,i, &self.positions, i);
            vecAdd(&mut self.positions,i, &self.velocities,i, dt);
            let y = self.positions[3 * i + 1];
            if y < 0.0 {
                vecCopy(&mut self.positions,i, &self.prev_positions,i);
                self.positions[3 * i + 1] = 0.0;
            }
        }
    }

    pub fn solve(&mut self, dt: f32)
    {
        self.solve_edges(self.edge_compliance, dt);
        self.solve_volumes(self.volume_compliance, dt);
    }

    pub fn post_solve(&mut self, dt: f32)
    {
        for i in 0..self.num_particles {
            if self.inv_mass[i] == 0.0 {
                continue;
            }
            vecSetDiff(&mut self.velocities,i, &self.positions,i, &self.prev_positions,i, 1.0 / dt);
        }
        // self.updateMeshes();
    }

    fn solve_edges(&mut self, compliance: f32, dt: f32) {
        let alpha = compliance / dt /dt;

        for i in 0..self.edge_lengths.len() {
            let id0 = self.edge_ids[2 * i];
            let id1 = self.edge_ids[2 * i + 1];
            let w0 = self.inv_mass[id0];
            let w1 = self.inv_mass[id1];
            let w = w0 + w1;
            if w == 0.0 {
                continue;
            }

            vecSetDiff(&mut self.grads,0, &self.positions,id0, &self.positions,id1, 1.0);
            let len = vecLengthSquared(&self.grads,0).sqrt();
            if len == 0.0 {
                continue;
            }
            vecScale(&mut self.grads,0, 1.0 / len);
            let restLen = self.edge_lengths[i];
            let C = len - restLen;
            let s = -C / (w + alpha);
            vecAdd(&mut self.positions,id0, &self.grads,0, s * w0);
            vecAdd(&mut self.positions,id1, &self.grads,0, -s * w1);
        }
    }

    fn solve_volumes(&mut self, compliance: f32, dt: f32) {
        let alpha = compliance / dt /dt;

        for i in 0..self.num_tets {
            let mut w = 0.0;
            
            for j in 0..4 {
                let id0 = self.tet_ids[4 * i + VOLUME_ID_ORDER[j][0]];
                let id1 = self.tet_ids[4 * i + VOLUME_ID_ORDER[j][1]];
                let id2 = self.tet_ids[4 * i + VOLUME_ID_ORDER[j][2]];

                vecSetDiff(&mut self.temp,0, &self.positions,id1, &self.positions,id0, 1.0);
                vecSetDiff(&mut self.temp,1, &self.positions,id2, &self.positions,id0, 1.0);
                vecSetCross(&mut self.grads,j, &self.temp,0, &self.temp,1);
                vecScale(&mut self.grads,j, 1.0/6.0);

                w += self.inv_mass[self.tet_ids[4 * i + j]] * vecLengthSquared(&self.grads,j);
            }
            if w == 0.0 {
                continue;
            }

            let vol = self.getTetVolume(i);
            let restVol = self.rest_volumn[i];
            let C = vol - restVol;
            let s = -C / (w + alpha);

            for j in 0..4 {
                let id = self.tet_ids[4 * i + j];
                vecAdd(&mut self.positions,id, &self.grads,j, s * self.inv_mass[id])
            }
        }
    }

    pub fn mesh_positions(&self) -> Vec<[f32; 3]> {
        self.positions
            .chunks_exact(3)
            .map(|v| {
                let x = v[0] as f32;
                let y = v[1] as f32;
                let z = v[2] as f32;
                [x, y, z]
            })
            .collect::<Vec<[f32; 3]>>()
    }

    pub fn mesh(&self) -> Mesh {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        let indices = self.indices.iter().map(|i| *i as u32).collect::<Vec<u32>>();
        mesh.set_indices(Some(Indices::U32(indices)));
        mesh.insert_attribute(
            Mesh::ATTRIBUTE_POSITION,
            self.positions
                .chunks_exact(3)
                .map(|v| {
                    let x = v[0] as f32;
                    let y = v[1] as f32;
                    let z = v[2] as f32;
                    [x, y, z]
                })
                .collect::<Vec<[f32; 3]>>(),
        );
        //mesh.insert_attribute(Mesh::ATTRIBUTE_NORMAL, normals);
        //mesh.insert_attribute(Mesh::ATTRIBUTE_UV_0, uvs);
        mesh.duplicate_vertices();
        mesh.compute_flat_normals();
        mesh
    }


}

fn vecSetZero(a: &mut Vec<f32>, anr: usize) {
    let ar = anr * 3;
    a[ar] = 0.0;
    a[ar + 1] = 0.0;
    a[ar + 2] = 0.0;
}

fn vecScale(a: &mut Vec<f32>, anr: usize, scale: f32) {
    let ar = anr * 3;
    a[ar] *= scale;
    a[ar + 1] *= scale;
    a[ar + 2] *= scale;
}

fn vecCopy(a: &mut [f32], anr: usize, b: &[f32], bnr: usize) {
    let ar = anr * 3;
    let br = bnr * 3;
    a[ar] = b[br];
    a[ar + 1] = b[br + 1];
    a[ar + 2] = b[br + 2];
}

fn vecAdd(a: &mut [f32], anr: usize, b: &[f32], bnr: usize, scale: f32) {
    let ar = anr * 3;
    let br = bnr * 3;
    a[ar] += b[br] * scale;
    a[ar + 1] += b[br + 1] * scale;
    a[ar + 2] += b[br + 2] * scale;
}

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

fn vecLengthSquared(a: &[f32], anr: usize) -> f32 {
    let ar = anr * 3;
    let a0 = a[ar];
    let a1 = a[ar + 1];
    let a2 = a[ar + 2];
    return a0 * a0 + a1 * a1 + a2 * a2;
}

fn vecDistSquared(a: &[f32], anr: usize, b: &[f32], bnr: usize) -> f32 {
    let ar = anr * 3;
    let br = bnr * 3;
    let a0 = a[ar] - b[br];
    let a1 = a[ar + 1] - b[br + 1];
    let a2 = a[ar + 2] - b[br + 2];
    return a0 * a0 + a1 * a1 + a2 * a2;
}

fn vecDot(a: &[f32], anr: usize, b: &[f32], bnr: usize) -> f32 {
    let ar = anr * 3;
    let br = bnr * 3;
    return a[ar] * b[br] + a[ar + 1] * b[br + 1] + a[ar + 2] * b[br + 2];
}

fn vecSetCross(a: &mut [f32], anr: usize, b: &[f32], bnr: usize, c: &[f32], cnr: usize) {
    let ar = anr * 3;
    let br = bnr * 3;
    let cr = cnr * 3;
    a[ar] = b[br + 1] * c[cr + 2] - b[br + 2] * c[cr + 1];
    a[ar + 1] = b[br + 2] * c[cr + 0] - b[br + 0] * c[cr + 2];
    a[ar + 2] = b[br + 0] * c[cr + 1] - b[br + 1] * c[cr + 0];
}
