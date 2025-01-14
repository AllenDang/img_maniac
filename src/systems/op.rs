use bevy::prelude::*;

use crate::shader::mat_separate_channel::MaterialSeparateChannel;

use super::rearrange::{DropInImage, EvtRearrange};

pub fn delete_all_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut cmds: Commands,
    q_image: Query<Entity, With<Sprite>>,
) {
    if keyboard_input.pressed(KeyCode::KeyX) {
        for entity in q_image.iter() {
            cmds.entity(entity).despawn_recursive();
        }
    }
}

pub fn manual_rearrage_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut evt_rearrage: EventWriter<EvtRearrange>,
) {
    if keyboard_input.pressed(KeyCode::KeyR) {
        evt_rearrage.send(EvtRearrange);
    }
}

pub fn change_channel_system(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut materials: ResMut<Assets<MaterialSeparateChannel>>,
    mut query: Query<&MeshMaterial2d<MaterialSeparateChannel>, With<DropInImage>>,
) {
    let mut change_channel = |ch: u32| {
        for mat_handle in query.iter_mut() {
            if let Some(mat) = materials.get_mut(mat_handle) {
                mat.channel = ch;
            }
        }
    };

    if keyboard_input.pressed(KeyCode::Digit1) {
        change_channel(1);
    }

    if keyboard_input.pressed(KeyCode::Digit2) {
        change_channel(2);
    }

    if keyboard_input.pressed(KeyCode::Digit3) {
        change_channel(3);
    }

    if keyboard_input.pressed(KeyCode::Digit4) {
        change_channel(4);
    }

    if keyboard_input.pressed(KeyCode::KeyA) {
        change_channel(0);
    }
}
