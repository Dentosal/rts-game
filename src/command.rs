use bevy::prelude::*;
use bevy_mod_raycast::Intersection;

use crate::MoveTarget;

use super::selection::SelectionInProgress;
use super::{CurrentPlayer, GroundRaycastSet, Player, Unit};

pub fn right_click(
    mut commands: Commands,
    windows: ResMut<Windows>,
    mouse_buttons: Res<Input<MouseButton>>,
    selection: Res<SelectionInProgress>,
    ground_intersect: Query<&Intersection<GroundRaycastSet>>,
) {
    let window = windows.get_primary().unwrap();

    let mut cursor_ground_pos = None;
    for intersection in ground_intersect.iter() {
        if let Some(pos) = intersection.position() {
            cursor_ground_pos = Some(*pos);
            break;
        }
    }

    let Some(cursor_ground_pos) = cursor_ground_pos else {
        return;
    };

    if mouse_buttons.just_released(MouseButton::Right) {
        for unit in &selection.current {
            // TODO: check that player is allowed to control these
            commands.entity(*unit).insert(MoveTarget(cursor_ground_pos));
        }
    }
}
