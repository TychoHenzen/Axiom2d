use axiom2d::prelude::{Entity, Resource};
use glam::Vec2;

#[derive(Resource, Debug)]
pub(crate) struct SelectedTerrain(pub(crate) usize);

#[derive(Resource, Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ViewerMode {
    SingleMaterial,
    DualGrid,
    WfcGrid,
}

#[derive(Resource, Debug)]
pub(crate) struct WfcState {
    pub(crate) seed: u64,
}

#[derive(Resource, Debug)]
pub(crate) struct TerrainQuadEntity(pub(crate) Entity);

#[derive(Resource, Debug)]
pub(crate) struct HudTextEntity(pub(crate) Entity);

#[derive(Resource, Debug, Default)]
pub(crate) struct GridTileEntities(pub(crate) Vec<Entity>);

#[derive(Resource, Debug, Default)]
pub(crate) struct CameraDragState {
    pub(crate) anchor: Option<Vec2>,
}

#[derive(Resource, Debug, Default)]
pub(crate) struct WfcGenerateRequested(pub(crate) bool);

#[derive(Resource, Debug, Default)]
pub(crate) struct ModeSwitchRequested(pub(crate) bool);
