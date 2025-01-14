use std::{path::Path, time::Duration};

use bevy::{asset::embedded_asset, prelude::*, sprite::Material2dPlugin, winit::WinitSettings};
use shader::mat_separate_channel::MaterialSeparateChannel;
use systems::{
    cam_control, file_drop, op,
    rearrange::{self, EvtRearrange},
};

mod shader;
mod systems;

fn main() {
    let mut app = App::new();
    app.insert_resource(WinitSettings {
        focused_mode: bevy::winit::UpdateMode::Continuous,
        unfocused_mode: bevy::winit::UpdateMode::reactive_low_power(Duration::from_secs(2)),
    })
    .add_plugins((
        DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: format!("Image Maniac v{}", env!("CARGO_PKG_VERSION")),
                resolution: (1440., 900.).into(),
                ..default()
            }),
            ..default()
        }),
        Material2dPlugin::<MaterialSeparateChannel>::default(),
    ))
    .add_event::<EvtRearrange>()
    .add_systems(Startup, setup)
    .add_systems(
        Update,
        (
            cam_control::cam_zoom_system,
            cam_control::cam_move_system,
            file_drop::file_drop_system,
            rearrange::rearrange_system.after(file_drop::file_drop_system),
            op::delete_all_system,
            op::manual_rearrage_system,
            op::change_channel_system,
        ),
    );

    embedded_asset!(app, "shader/separate_channel.wgsl");

    app.run();
}

fn setup(
    mut commands: Commands,
    mut evt_file_drag_and_drop: EventWriter<FileDragAndDrop>,
    q_win: Query<Entity, With<Window>>,
) {
    commands.spawn((
        Camera2d,
        systems::cam_control::CamStatus {
            target_scale: 2.0,
            current_scale: 2.0,
        },
    ));

    commands.spawn((
        Text::new("A: Show RGBA | 1-4: Switch RGBA | R: Re-arrange | X: Del All | MousWheel: Zoom | MMB: Pan"),
        TextFont {
            font_size: 14.0,
            ..default()
        },
        TextColor(Color::from(bevy::color::palettes::css::GRAY)),
        TextLayout::new_with_justify(JustifyText::Center),
        Node {
            bottom: Val::Px(10.0),
            position_type: PositionType::Absolute,
            width: Val::Percent(100.),
            ..default()
        },
    ));

    // Process command line args, should be multiple image paths
    let args: Vec<String> = std::env::args().skip(1).collect();
    if !args.is_empty() {
        if let Ok(cwd) = std::env::current_dir() {
            for arg in args.iter() {
                let img_path = if Path::new(arg).is_relative() {
                    cwd.join(arg)
                } else {
                    Path::new(arg).to_path_buf()
                };

                if img_path.exists() && file_drop::is_supported_format(&img_path) {
                    evt_file_drag_and_drop.send(FileDragAndDrop::DroppedFile {
                        window: q_win.single(),
                        path_buf: img_path,
                    });
                }
            }
        }
    }
}
