use crate::card::rendering::geometry::{TABLE_CARD_HEIGHT, TABLE_CARD_WIDTH};
use crate::stash::constants::{SLOT_HEIGHT, SLOT_WIDTH};

pub(crate) fn miniature_card_model(zoom: f32, center_x: f32, center_y: f32) -> [[f32; 4]; 4] {
    let scale_x = (SLOT_WIDTH / zoom) / TABLE_CARD_WIDTH;
    let scale_y = (SLOT_HEIGHT / zoom) / TABLE_CARD_HEIGHT;
    [
        [scale_x, 0.0, 0.0, 0.0],
        [0.0, scale_y, 0.0, 0.0],
        [0.0, 0.0, 1.0, 0.0],
        [center_x, center_y, 0.0, 1.0],
    ]
}
