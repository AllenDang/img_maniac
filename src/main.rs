use std::path::{Path, PathBuf};

use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    winit::WinitSettings,
};
use bevy_mod_picking::{
    InteractablePickingPlugin, PickableBundle, PickingCameraBundle, PickingEvent, PickingPlugin,
    Selection, SelectionEvent,
};

use check_img_format::is_supported_format;
use mat_separate_channel::MaterialSeparateChannel;
use taffy::style_helpers::{TaffyAuto, TaffyMaxContent};

mod check_img_format;
mod mat_separate_channel;

struct ImageDropEvent {
    dropped_image_path: PathBuf,
    world_pos: Vec2,
}

struct RearrangeEvent;

#[derive(Component)]
struct CameraController;

#[derive(Component)]
struct DropInImage;

#[derive(Component)]
struct Resized;

const QUAD_SIZE: f32 = 3.0;

fn main() {
    App::new()
        .insert_resource(WinitSettings::desktop_app())
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Image Maniac".into(),
                resolution: (1440., 900.).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugin(MaterialPlugin::<MaterialSeparateChannel>::default())
        .add_plugin(PickingPlugin)
        .add_plugin(InteractablePickingPlugin)
        .add_event::<ImageDropEvent>()
        .add_event::<RearrangeEvent>()
        .add_startup_system(startup_system)
        .add_systems((
            file_drag_and_drop_system,
            image_dropped_system,
            update_quad_ratio_system,
            change_channel_system,
            camera_control_system,
            change_cursor_system,
            delete_selections_system,
            highlight_outline_system,
            drag_move_system,
            rearrange_image_system,
            trigger_rearrange_system,
        ))
        .run()
}

fn startup_system(
    mut cmds: Commands,
    mut image_drop_event_writer: EventWriter<ImageDropEvent>,
    asset_server: Res<AssetServer>,
) {
    // camera
    cmds.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 0.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        CameraController,
        PickingCameraBundle::default(),
    ));

    // UI
    let font_handle = asset_server.load("font/FiraCode-Regular.ttf");

    let panel_style = Style {
        align_self: AlignSelf::FlexEnd,
        justify_content: JustifyContent::Center,
        align_items: AlignItems::Center,
        position: UiRect {
            bottom: Val::Px(10.0),
            ..Default::default()
        },
        size: Size::new(Val::Percent(100.0), Val::Auto),
        ..Default::default()
    };

    let text_style = TextStyle {
        font: font_handle,
        font_size: 16.0,
        color: Color::GRAY,
    };

    cmds.spawn(NodeBundle {
        style: panel_style,
        ..Default::default()
    })
    .with_children(|parent| {
        parent.spawn(TextBundle::from_section(
            "A: Show RGBA | 1-4: Switch RGBA | R: Re-arrange | MouseWheel: Zoom | Space+LMB: Move Canvas | ESC: Reset | X: Del Selected | Shift+X: Del All",
            text_style,
        ));
    });

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

                if img_path.exists() && is_supported_format(&img_path) {
                    image_drop_event_writer.send(ImageDropEvent {
                        dropped_image_path: img_path,
                        world_pos: Vec2::ZERO,
                    });
                }
            }
        }
    }
}

fn file_drag_and_drop_system(
    mut file_events: EventReader<FileDragAndDrop>,
    mut image_drop_event_writer: EventWriter<ImageDropEvent>,
    windows: Query<&mut Window>,
    cameras: Query<(&Camera, &GlobalTransform), With<CameraController>>,
) {
    let (cam, cam_transform) = cameras.single();

    for event in file_events.iter() {
        if let FileDragAndDrop::DroppedFile { window, path_buf } = event {
            let win = windows.get(*window).unwrap();

            if is_supported_format(path_buf) {
                let mut world_pos: Option<Vec2> = None;

                if let Some(ray) = win
                    .cursor_position()
                    .and_then(|cursor| cam.viewport_to_world(cam_transform, cursor))
                {
                    if let Some(distance) = ray.intersect_plane(Vec3::ZERO, Vec3::Z) {
                        world_pos = Some(ray.get_point(distance).truncate());
                    }
                }

                image_drop_event_writer.send(ImageDropEvent {
                    dropped_image_path: path_buf.clone(),
                    world_pos: if let Some(world_pos) = world_pos {
                        world_pos
                    } else {
                        Vec2::ZERO
                    },
                })
            }
        }
    }
}

fn change_cursor_system(
    keyboard_input: Res<Input<KeyCode>>,
    mouse_input: Res<Input<MouseButton>>,
    mut windows: Query<&mut Window>,
) {
    let mut window = windows.single_mut();

    let space = keyboard_input.pressed(KeyCode::Space);
    let mouse_left = mouse_input.pressed(MouseButton::Left);

    window.cursor.icon = match (space, mouse_left) {
        (true, false) => CursorIcon::Grab,
        (true, true) => CursorIcon::Grabbing,
        _ => CursorIcon::Default,
    };
}

fn camera_control_system(
    keyboard_input: Res<Input<KeyCode>>,
    mouse_input: Res<Input<MouseButton>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut query: Query<&mut Transform, With<CameraController>>,
) {
    // Space + LMB to move camera
    if keyboard_input.pressed(KeyCode::Space) && mouse_input.pressed(MouseButton::Left) {
        let mut delta: Vec2 = Vec2::ZERO;
        for event in mouse_motion_events.iter() {
            delta += event.delta;
        }

        if delta != Vec2::ZERO {
            for mut transform in query.iter_mut() {
                transform.translation.x -= delta.x * 0.01;
                transform.translation.y += delta.y * 0.01;
            }
        }
    }

    // Escape to reset camera
    if keyboard_input.just_pressed(KeyCode::Escape) {
        for mut transform in query.iter_mut() {
            transform.translation = Vec3::new(0.0, 0.0, 8.0);
        }
    }

    // Handle mouse wheel input to translate camera's z position
    for event in mouse_wheel_events.iter() {
        for mut transform in query.iter_mut() {
            let new_z = (transform.translation.z - event.y * 0.1).clamp(1.0, 30.0);

            if new_z != 0.0 {
                transform.translation.z = new_z;
            }
        }
    }
}

fn image_dropped_system(
    mut image_drop_event_reader: EventReader<ImageDropEvent>,
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<MaterialSeparateChannel>>,
    imgs: Query<&DropInImage>,
    asset_server: Res<AssetServer>,
) {
    let mut img_count = imgs.iter().count() + 1;

    for evt in image_drop_event_reader.iter() {
        // Check the maximum width and height before createing a texture
        // wgpu has a limit of 16384x16384
        if let Ok(dim) = imagesize::size(evt.dropped_image_path.clone()) {
            if dim.width >= 16384 || dim.height >= 16384 {
                //TODO: Show error message
                continue;
            }
        }

        let tex_handle = asset_server.load(evt.dropped_image_path.clone());
        let quad_handle = meshes.add(Mesh::from(shape::Quad::new(Vec2::new(
            QUAD_SIZE, QUAD_SIZE,
        ))));

        let mat_handle = materials.add(MaterialSeparateChannel {
            base_color_texture: Some(tex_handle),
            channel: 0,
            show_outline: 0,
            outline_color: Color::YELLOW,
            outline_width: 1.0,
        });

        cmds.spawn((
            MaterialMeshBundle {
                mesh: quad_handle,
                material: mat_handle,
                transform: Transform::from_xyz(
                    evt.world_pos.x,
                    evt.world_pos.y,
                    img_count as f32 / 10.0,
                ),
                ..default()
            },
            PickableBundle::default(),
            DropInImage,
        ));

        img_count += 1;
    }
}

#[allow(clippy::type_complexity)]
fn update_quad_ratio_system(
    mut cmds: Commands,
    mut query: Query<
        (Entity, &Handle<MaterialSeparateChannel>, &mut Transform),
        (With<DropInImage>, Without<Resized>),
    >,
    materials: Res<Assets<MaterialSeparateChannel>>,
    images: Res<Assets<Image>>,
) {
    for (entity, mat_handle, mut transform) in query.iter_mut() {
        if let Some(mat) = materials.get(mat_handle) {
            if let Some(tex) = images.get(mat.base_color_texture.as_ref().unwrap()) {
                let ratio = tex.size().x / tex.size().y;
                transform.scale = Vec3::new(ratio, 1.0, 1.0);

                cmds.entity(entity).insert(Resized);
            }
        }
    }
}

fn change_channel_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut materials: ResMut<Assets<MaterialSeparateChannel>>,
    mut query: Query<&Handle<MaterialSeparateChannel>, With<DropInImage>>,
) {
    let mut change_channel = |ch: u32| {
        for mat_handle in query.iter_mut() {
            if let Some(mat) = materials.get_mut(mat_handle) {
                mat.channel = ch;
            }
        }
    };

    if keyboard_input.pressed(KeyCode::Key1) {
        change_channel(1);
    }

    if keyboard_input.pressed(KeyCode::Key2) {
        change_channel(2);
    }

    if keyboard_input.pressed(KeyCode::Key3) {
        change_channel(3);
    }

    if keyboard_input.pressed(KeyCode::Key4) {
        change_channel(4);
    }

    if keyboard_input.pressed(KeyCode::A) {
        change_channel(0);
    }
}

fn delete_selections_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut cmds: Commands,
    query: Query<(Entity, &Selection)>,
) {
    // press x to delete selected image
    if keyboard_input.pressed(KeyCode::X) && !keyboard_input.pressed(KeyCode::LShift) {
        for (entity, sel) in query.iter() {
            if sel.selected() {
                cmds.entity(entity).despawn_recursive();
            }
        }
    }

    // press shift + x to delete all
    if keyboard_input.pressed(KeyCode::X) && keyboard_input.pressed(KeyCode::LShift) {
        for (entity, _) in query.iter() {
            cmds.entity(entity).despawn_recursive();
        }
    }
}

fn highlight_outline_system(
    mut materials: ResMut<Assets<MaterialSeparateChannel>>,
    mut query_mat: Query<&Handle<MaterialSeparateChannel>>,
    mut events: EventReader<PickingEvent>,
) {
    for event in events.iter() {
        if let PickingEvent::Selection(s) = event {
            match s {
                SelectionEvent::JustSelected(s) => {
                    if let Ok(mat_handle) = query_mat.get_mut(*s) {
                        if let Some(mat) = materials.get_mut(mat_handle) {
                            mat.show_outline = 1;
                        }
                    }
                }
                SelectionEvent::JustDeselected(s) => {
                    if let Ok(mat_handle) = query_mat.get_mut(*s) {
                        if let Some(mat) = materials.get_mut(mat_handle) {
                            mat.show_outline = 0;
                        }
                    }
                }
            }
        }
    }
}

fn drag_move_system(
    keyboard_input: Res<Input<KeyCode>>,
    mouse_input: Res<Input<MouseButton>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut query: Query<(&Selection, &mut Transform)>,
) {
    if mouse_input.pressed(MouseButton::Left) && !keyboard_input.pressed(KeyCode::Space) {
        let mut delta: Vec2 = Vec2::ZERO;
        for event in mouse_motion_events.iter() {
            delta += event.delta;
        }

        if delta != Vec2::ZERO {
            for (sel, mut transform) in query.iter_mut() {
                if sel.selected() {
                    transform.translation.x += delta.x * 0.01;
                    transform.translation.y -= delta.y * 0.01;
                }
            }
        }
    }
}

fn rearrange_image_system(
    mut events: EventReader<RearrangeEvent>,
    mut query: Query<&mut Transform, With<Resized>>,
) {
    if events.iter().len() > 0 && query.iter().len() > 0 {
        let mut t = taffy::Taffy::new();
        let mut nodes = Vec::new();
        let mut entities = Vec::new();

        let factor = 100.0;

        for trans in query.iter_mut() {
            let node = t
                .new_leaf(taffy::prelude::Style {
                    size: taffy::prelude::Size {
                        width: taffy::prelude::Dimension::Points(
                            trans.scale.x * QUAD_SIZE * factor,
                        ),
                        height: taffy::prelude::Dimension::Points(
                            trans.scale.y * QUAD_SIZE * factor,
                        ),
                    },
                    ..default()
                })
                .unwrap();

            nodes.push(node);
            entities.push(trans);
        }

        let max_width = 7.0 * QUAD_SIZE * factor;

        let root = t
            .new_with_children(
                taffy::prelude::Style {
                    flex_direction: taffy::prelude::FlexDirection::Row,
                    flex_wrap: taffy::prelude::FlexWrap::Wrap,
                    justify_content: Some(taffy::prelude::JustifyContent::Center),
                    align_items: Some(taffy::prelude::AlignItems::Center),
                    gap: taffy::prelude::Size {
                        width: taffy::prelude::LengthPercentage::Points(0.16 * factor),
                        height: taffy::prelude::LengthPercentage::Points(0.16 * factor),
                    },
                    size: taffy::prelude::Size {
                        width: taffy::prelude::Dimension::Points(max_width),
                        height: taffy::prelude::Dimension::AUTO,
                    },
                    ..default()
                },
                &nodes,
            )
            .unwrap();

        t.compute_layout(root, taffy::prelude::Size::MAX_CONTENT)
            .unwrap();

        for (i, n) in nodes.iter().enumerate() {
            if let Ok(n) = t.layout(*n) {
                entities[i].translation = Vec3::new(
                    (n.location.x - max_width / 2.0) / factor,
                    n.location.y / factor,
                    entities[i].translation.z,
                )
            }
        }
    }
}

fn trigger_rearrange_system(
    keyboard_input: Res<Input<KeyCode>>,
    mut rearrange_event: EventWriter<RearrangeEvent>,
) {
    if keyboard_input.pressed(KeyCode::R) {
        rearrange_event.send(RearrangeEvent);
    }
}
