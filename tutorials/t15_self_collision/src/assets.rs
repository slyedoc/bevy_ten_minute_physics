use bevy::{
    asset::{AssetLoader, LoadContext, LoadedAsset},
    prelude::*,
    reflect::TypeUuid,
    utils::BoxedFuture,
};
use serde::Deserialize;
use serde_json::from_slice;

pub struct MeshAssetsPlugin;

impl Plugin for MeshAssetsPlugin {
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
