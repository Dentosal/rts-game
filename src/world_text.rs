use bevy::prelude::*;

use super::PrimaryCamera;

/// UI text of a world-entity
#[derive(Component)]
pub struct WorldText(pub Entity);

#[derive(Component)]
pub struct WorldTextBacklink(pub Entity);

pub fn sync_world_text_position(
    windows: Res<Windows>,
    images: Res<Assets<Image>>,
    mut text_query: Query<(&mut Style, &CalculatedSize, &mut Visibility, &WorldText)>,
    pos_query: Query<&Transform>,
    camera_query: Query<(&Camera, &GlobalTransform), With<PrimaryCamera>>,
) {
    for (camera, camera_transform) in camera_query.iter() {
        for (mut style, calc, mut visibility, WorldText(world_text)) in text_query.iter_mut() {
            if let Ok(pos) = pos_query.get(*world_text) {
                match camera.world_to_screen(&windows, &images, camera_transform, pos.translation) {
                    Some(coords) => {
                        style.position.left = Val::Px(coords.x - calc.size.width / 2.0);
                        style.position.bottom = Val::Px(coords.y + calc.size.height / 2.0);
                        visibility.is_visible = true;
                    }
                    None => {
                        visibility.is_visible = false;
                    }
                }
            }
        }
    }
}
