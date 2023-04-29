use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
    utils::BoxedFuture, render::{render_resource::PrimitiveTopology, mesh::Indices},
};
use serde::Deserialize;
use serde_json::from_slice;

pub struct TetMeshPlugin;

impl Plugin for TetMeshPlugin {
    fn build(&self, app: &mut App) {
        app.add_asset::<TetMesh>()
            .init_asset_loader::<TetMeshLoader>();
    }
}

#[derive(Debug, Deserialize, TypeUuid, Clone)]
#[uuid = "39cadc56-aa9c-4543-8640-a018b74b5052"]
pub struct TetMesh {
    pub vertices: Vec<f32>,
    pub indices: Vec<usize>,

    pub tet_vertices: Vec<f32>,
    pub tet_indices: Vec<usize>,
    pub tet_edge_ids: Vec<usize>,
}

impl From<&TetMesh> for Mesh {
    fn from(tet_mesh: &TetMesh) -> Self {
        let mut mesh = Mesh::new(PrimitiveTopology::TriangleList);
        mesh.insert_attribute(Mesh::ATTRIBUTE_POSITION, tet_mesh.vertices.chunks_exact(3)
        .map(|v| [v[0], v[1], v[2]])
        .collect::<Vec<[f32; 3]>>());
        mesh.set_indices(Some(Indices::U32(tet_mesh.indices.iter().map(|&i| i as u32).collect())));
        mesh.duplicate_vertices();
        mesh.compute_flat_normals();
        mesh
    }
}

#[derive(Default)]
pub struct TetMeshLoader;

impl AssetLoader for TetMeshLoader {
    fn load<'a>(
        &'a self,
        bytes: &'a [u8],
        load_context: &'a mut LoadContext,
    ) -> BoxedFuture<'a, Result<(), bevy::asset::Error>> {
        Box::pin(async move {
            let custom_asset = from_slice::<TetMesh>(bytes)?;
            load_context.set_default_asset(LoadedAsset::new(custom_asset));
            Ok(())
        })
    }

    fn extensions(&self) -> &[&str] {
        &["tet.json"]
    }
}
