use bevy::prelude::*;

use crate::MoveTarget;

use super::world_text::{WorldText, WorldTextBacklink};
use super::{Health, Unit};

pub fn set_status_text(
    units: Query<(&WorldTextBacklink, &Health), With<Unit>>,
    mut texts: Query<&mut Text>,
) {
    for (WorldTextBacklink(text_link), health) in units.iter() {
        let mut text = texts.get_mut(*text_link).unwrap();
        dbg!(text, health);
    }
}

pub fn set_status_text2(
    mut texts: Query<(&WorldText, &mut Text)>,
    units: Query<&Health, With<Unit>>,
) {
    for (WorldText(link), mut text) in texts.iter_mut() {
        if let Ok(health) = units.get(*link) {
            text.sections[0].value = format!("{}/{}", health.current, health.max);
        }
    }
}

pub fn move_towards_target(
    time: Res<Time>,
    mut units: Query<(&mut Transform, &MoveTarget), With<Unit>>,
) {
    for (mut tr, MoveTarget(target)) in units.iter_mut() {
        let mut target = *target;
        target.y = 0.1;
        let to_target = target - tr.translation;
        if to_target.length() > 0.01 {
            let n = to_target.normalize();
            tr.translation = tr.translation + n * time.delta_seconds();
            let mut looking = tr.clone();
            looking.look_at(target, Vec3::Y);
            tr.rotation = tr.rotation.lerp(looking.rotation, 0.1);
        } else {
            tr.translation = target;
        }
    }
}
