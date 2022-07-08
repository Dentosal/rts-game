use bevy::prelude::*;
use bevy_mod_raycast::Intersection;

use super::{CurrentPlayer, GroundRaycastSet, Player, Unit};

#[derive(Debug, Default)]
pub struct SelectionInProgress {
    box_start: Option<CursorRaycast>,
    pub current: Vec<Entity>,
}

#[derive(Debug, Clone, Copy)]
pub struct CursorRaycast {
    screen: Vec2,
    world: Vec3,
}

#[derive(Component)]
pub struct SelectionBox;

pub fn setup(mut commands: Commands) {
    commands.insert_resource(SelectionInProgress::default());

    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                border: Rect::all(Val::Px(2.0)),
                ..default()
            },
            color: Color::rgba(0.4, 0.4, 1.0, 0.2).into(),
            visibility: Visibility { is_visible: false },
            ..default()
        })
        .insert(SelectionBox);
}

pub fn left_click(
    windows: ResMut<Windows>,
    mouse_buttons: Res<Input<MouseButton>>,
    mut selection: ResMut<SelectionInProgress>,
    mut selection_box: Query<(&mut Style, &mut Visibility), With<SelectionBox>>,
    ground_intersect: Query<&Intersection<GroundRaycastSet>>,
    players: Query<(Entity, &Children, Option<&CurrentPlayer>), With<Player>>,
    player_units: Query<(Entity, &GlobalTransform), With<Unit>>,
) {
    let window = windows.get_primary().unwrap();
    let Some(cursor_position) = window.cursor_position() else {
        return;
    };

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

    if mouse_buttons.just_released(MouseButton::Left) {
        if let Some(start) = selection.box_start.take() {
            let (_, mut visibility) = selection_box.single_mut();
            visibility.is_visible = false;

            selection.current.clear(); // TODO: when shift pressed, do not clear

            for (_player_entity, children, current) in players.iter() {
                // TODO: allow selecting non-owned units as well
                if !current.is_some() {
                    continue;
                }

                for &child in children.iter() {
                    let (unit_entity, position) = player_units.get(child).unwrap();
                    let unit_x = position.translation.x;
                    let unit_z = position.translation.z;
                    let min_x = start.world.x.min(cursor_ground_pos.x);
                    let max_x = start.world.x.max(cursor_ground_pos.x);
                    let min_z = start.world.z.min(cursor_ground_pos.z);
                    let max_z = start.world.z.max(cursor_ground_pos.z);
                    let in_x = min_x <= unit_x && unit_x <= max_x;
                    let in_z = min_z <= unit_z && unit_z <= max_z;
                    if in_x && in_z {
                        selection.current.push(unit_entity);
                    }
                }
            }
        }
    } else if mouse_buttons.pressed(MouseButton::Left) {
        if selection.box_start.is_none() {
            selection.box_start = Some(CursorRaycast {
                screen: cursor_position,
                world: cursor_ground_pos,
            });
        }

        let start = selection.box_start.as_ref().unwrap().screen;
        let end = cursor_position;
        let (mut selbox, mut visibility) = selection_box.single_mut();
        let box_w = (start.x - end.x).abs();
        let box_h = (start.y - end.y).abs();
        let box_x = start.x.min(end.x);
        let box_y = start.y.min(end.y);
        selbox.position = Rect {
            left: Val::Px(box_x),
            right: Val::Px(window.width() - (box_x + box_w)),
            top: Val::Px(window.height() - (box_y + box_h)),
            bottom: Val::Px(box_y),
        };
        visibility.is_visible = true;
    }
}
