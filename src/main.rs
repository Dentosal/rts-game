#![feature(let_else)]

use bevy::prelude::*;
use bevy_mod_raycast::{
    DefaultPluginState, DefaultRaycastingPlugin, RayCastMesh, RayCastMethod, RayCastSource,
    RaycastSystem,
};

pub mod command;
pub mod selection;
pub mod unit;
pub mod world_text;

/// A player, i.e. owner of a faction
#[derive(Component, Debug)]
pub struct Player;

/// Player who is playing on this instance
#[derive(Component, Debug)]
pub struct CurrentPlayer;

/// Player nickname
#[derive(Component, Debug)]
pub struct Nick(String);

/// Player-owned game entity
#[derive(Component)]
pub struct Unit;

/// Player-owned game entity
#[derive(Component, Debug)]
pub struct Health {
    current: u32,
    max: u32,
}

#[derive(Component)]
pub struct MoveTarget(Vec3);

#[derive(Component)]
pub struct Tail;

#[derive(Component)]
pub struct PrimaryCamera;

/// Raycast for ground plane(s)
pub struct GroundRaycastSet;

/// Raycast for units
pub struct UnitRaycastSet;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugin(DefaultRaycastingPlugin::<GroundRaycastSet>::default())
        .add_plugin(DefaultRaycastingPlugin::<UnitRaycastSet>::default())
        .add_startup_system(camera_setup)
        .add_startup_system(scene_setup)
        .add_startup_system(add_players)
        .add_system_to_stage(
            CoreStage::First,
            update_raycast_with_cursor
                .before(RaycastSystem::BuildRays::<GroundRaycastSet>)
                .before(RaycastSystem::BuildRays::<UnitRaycastSet>),
        )
        .add_startup_system(selection::setup)
        // .add_system(selection_box_update)
        .add_system(selection::left_click)
        .add_system(command::right_click)
        .add_system(world_text::sync_world_text_position)
        .add_system(unit::set_status_text)
        .add_system(unit::set_status_text2)
        .add_system(unit::move_towards_target)
        .add_system(rotate_camera)
        .add_system(wag_tail)
        .run();
}

fn wag_tail(time: Res<Time>, mut query: Query<&mut Transform, With<Tail>>) {
    for mut tr in query.iter_mut() {
        let t = time.seconds_since_startup() * 4.0;
        let t = t.sin() * 0.5;
        let x = t.sin() as f32;
        let z = t.cos() as f32;
        let tt = Transform::from_xyz(x, 0.0, z).looking_at(Vec3::ZERO, Vec3::Y);
        tr.rotation = tt.rotation;
    }
}

fn rotate_camera(time: Res<Time>, mut query: Query<&mut Transform, With<PrimaryCamera>>) {
    for mut transform in query.iter_mut() {
        let t = time.seconds_since_startup() * 0.1;
        let x = t.sin() as f32 * 5.0;
        let z = t.cos() as f32 * 5.0;
        *transform = Transform::from_xyz(x, 2.5, z).looking_at(Vec3::ZERO, Vec3::Y)
    }
}

fn camera_setup(mut commands: Commands) {
    commands.insert_resource(DefaultPluginState::<UnitRaycastSet>::default().with_debug_cursor());

    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_xyz(-2.0, 2.5, 5.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        })
        .insert(PrimaryCamera)
        .insert(RayCastSource::<UnitRaycastSet>::new_transform_empty())
        .insert(RayCastSource::<GroundRaycastSet>::new_transform_empty());

    commands.spawn_bundle(UiCameraBundle::default());
}

fn update_raycast_with_cursor(
    mut cursor: EventReader<CursorMoved>,
    mut grounds: Query<&mut RayCastSource<GroundRaycastSet>>,
    mut units: Query<&mut RayCastSource<UnitRaycastSet>>,
) {
    // Grab the most recent cursor event if it exists:
    let cursor_position = match cursor.iter().last() {
        Some(cursor_moved) => cursor_moved.position,
        None => return,
    };

    for mut pick_source in &mut grounds.iter_mut() {
        pick_source.cast_method = RayCastMethod::Screenspace(cursor_position);
    }

    for mut pick_source in &mut units.iter_mut() {
        pick_source.cast_method = RayCastMethod::Screenspace(cursor_position);
    }
}

fn add_players(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: ResMut<AssetServer>,
) {
    let font = asset_server.load("fonts/PublicPixel.ttf");

    let player_ids = vec![
        commands
            .spawn()
            .insert(GlobalTransform::identity())
            .insert(Transform::identity())
            .insert(Player)
            .insert(CurrentPlayer)
            .insert(Nick("Dentosal".to_string()))
            .id(),
        commands
            .spawn()
            .insert(GlobalTransform::identity())
            .insert(Transform::identity())
            .insert(Player)
            .insert(Nick("Opponent".to_string()))
            .id(),
    ];

    for (index, player) in player_ids.into_iter().enumerate() {
        let eye1 = commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: 0.03 })),
                material: materials.add(Color::rgb(1.0, 0.0, 0.0).into()),
                transform: Transform::from_xyz(-0.025, 0.005, -0.1),
                ..default()
            })
            .id();

        let eye2 = commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: 0.03 })),
                material: materials.add(Color::rgb(1.0, 0.0, 0.0).into()),
                transform: Transform::from_xyz(0.025, 0.005, -0.1),
                ..default()
            })
            .id();

        let tail = commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Box {
                    min_x: -0.02,
                    max_x: 0.02,
                    min_y: -0.02,
                    max_y: 0.02,
                    min_z: 0.0,
                    max_z: 0.2,
                })),
                material: materials.add(Color::rgb(0.0, 0.0, 1.0).into()),
                transform: Transform::from_xyz(0.0, -0.08, 0.1),
                ..default()
            })
            .insert(Tail)
            .id();

        let child = commands
            .spawn_bundle(PbrBundle {
                mesh: meshes.add(Mesh::from(shape::Cube { size: 0.2 })),
                material: materials.add(Color::rgb(0.8, 0.7, 0.6).into()),
                transform: Transform::from_xyz(if index == 0 { -1.0 } else { 1.0 }, 0.1, 0.0),
                ..default()
            })
            .insert(Unit)
            .insert(Health {
                current: ((index + 1) * 50) as u32,
                max: 100,
            })
            .insert(RayCastMesh::<UnitRaycastSet>::default())
            .push_children(&[eye1, eye2, tail])
            .id();

        let child_text = commands
            .spawn_bundle(TextBundle {
                style: Style {
                    align_self: AlignSelf::Center,
                    position_type: PositionType::Absolute,
                    ..Default::default()
                },
                text: Text::with_section(
                    "A Cube".to_string(),
                    TextStyle {
                        font: font.clone(),
                        font_size: 18.0,
                        color: Color::WHITE,
                    },
                    TextAlignment {
                        ..Default::default()
                    },
                ),
                ..Default::default()
            })
            .insert(world_text::WorldText(child))
            .id();

        commands
            .entity(player)
            .insert(world_text::WorldTextBacklink(child_text))
            .push_children(&[child]);
    }
}

fn scene_setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // plane
    commands
        .spawn_bundle(PbrBundle {
            mesh: meshes.add(Mesh::from(shape::Plane { size: 5.0 })),
            material: materials.add(Color::rgb(0.3, 0.5, 0.3).into()),
            ..default()
        })
        .insert(RayCastMesh::<GroundRaycastSet>::default());

    // light
    commands.spawn_bundle(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
}
