#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use std::path::{Path, PathBuf};

use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    render::camera::ScalingMode,
    winit::WinitSettings,
};
use bevy_mod_picking::{picking_core::PickingPluginsSettings, prelude::*};
use check_img_format::is_supported_format;
use mat_progress_indicator::MaterialProgressIndicator;
use mat_separate_channel::MaterialSeparateChannel;
use taffy::style_helpers::{TaffyAuto, TaffyMaxContent};

mod check_img_format;
mod mat_progress_indicator;
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

const QUAD_SIZE: f32 = 3.0;
const ZOOM_SPEED: f32 = 1.1;
const DRAG_SPEED: f32 = 0.002;

fn main() {
    App::new()
        .insert_resource(WinitSettings::desktop_app())
        .insert_resource(ClearColor(Color::rgb(0.1, 0.1, 0.1)))
        .insert_resource(PickingPluginsSettings {
            enable: true,
            enable_input: true,
            enable_highlighting: true,
            enable_interacting: true,
        })
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: format!("Image Maniac v{}", env!("CARGO_PKG_VERSION")),
                resolution: (1440., 900.).into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugin(MaterialPlugin::<MaterialSeparateChannel>::default())
        .add_plugin(MaterialPlugin::<MaterialProgressIndicator>::default())
        .add_plugins(DefaultPickingPlugins)
        .add_event::<ImageDropEvent>()
        .add_event::<RearrangeEvent>()
        .add_startup_system(startup_system)
        .add_systems((
            file_drag_and_drop_system,
            image_dropped_system,
            change_channel_system,
            camera_control_system,
            change_cursor_system,
            delete_selections_system,
            highlight_outline_system,
            rearrange_image_system,
            trigger_rearrange_system,
            create_loading_progress,
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
            transform: Transform::from_xyz(0.0, 0.0, 3.0).looking_at(Vec3::ZERO, Vec3::Y),
            projection: OrthographicProjection {
                scale: 3.0,
                scaling_mode: ScalingMode::FixedVertical(2.0),
                ..default()
            }
            .into(),
            ..default()
        },
        CameraController,
        RaycastPickCamera::default(),
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

fn create_loading_progress(
    keyboard_input: Res<Input<KeyCode>>,
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<MaterialProgressIndicator>>,
) {
    if keyboard_input.pressed(KeyCode::A) {
        cmds.spawn(MaterialMeshBundle {
            mesh: meshes.add(Mesh::from(shape::Quad::new(Vec2::new(
                QUAD_SIZE + 1.0,
                QUAD_SIZE,
            )))),
            material: materials.add(MaterialProgressIndicator {
                background_color: Color::BLUE,
                aspect_ratio: (QUAD_SIZE + 1.0) / QUAD_SIZE,
                ring_inner_radius: 0.4,
                ring_outer_radius: 0.45,
                ring_color: Color::WHITE,
                nob_radius: 0.1,
                nob_color: Color::RED,
                nob_rotation_speed: 5.0,
                dot_radius: 0.03,
                dot_color: Color::YELLOW,
                dot_distance: 0.1,
                dot_margin_top: -0.2,
                dot_fade_speed: 2.0,
                anti_alias_factor: 0.01,
            }),
            ..default()
        });
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

#[allow(clippy::too_many_arguments)]
fn camera_control_system(
    keyboard_input: Res<Input<KeyCode>>,
    mouse_input: Res<Input<MouseButton>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut picking_setting: ResMut<PickingPluginsSettings>,
    mut query: Query<(&mut Transform, &mut Projection), With<CameraController>>,
    mut materials: ResMut<Assets<MaterialSeparateChannel>>,
    mut query_mat: Query<&Handle<MaterialSeparateChannel>, With<PickSelection>>,
) {
    let (mut cam_transform, mut projection) = query.single_mut();

    if keyboard_input.just_pressed(KeyCode::Space) {
        picking_setting.enable = false;
    }

    if keyboard_input.just_released(KeyCode::Space) {
        picking_setting.enable = true;
    }

    // Space + LMB to move camera
    if keyboard_input.pressed(KeyCode::Space) && mouse_input.pressed(MouseButton::Left) {
        let mut delta: Vec2 = Vec2::ZERO;
        for event in mouse_motion_events.iter() {
            delta += event.delta;
        }

        let speed = 0.0025;

        if delta != Vec2::ZERO {
            if let Projection::Orthographic(ortho) = projection.as_mut() {
                cam_transform.translation.x -= delta.x * speed * ortho.scale;
                cam_transform.translation.y += delta.y * speed * ortho.scale;
            }
        }
    }

    // Escape to reset camera
    if keyboard_input.just_pressed(KeyCode::Escape) {
        cam_transform.translation = Vec3::new(0.0, 0.0, 3.0);
    }

    // Handle mouse wheel input to translate camera's z position
    if let Projection::Orthographic(ortho) = projection.as_mut() {
        for event in mouse_wheel_events.iter() {
            let mut scale = ortho.scale;
            scale *= if event.y <= 0.0 {
                ZOOM_SPEED
            } else {
                1.0 / ZOOM_SPEED
            };

            ortho.scale = scale.clamp(0.1, 12.0);
        }

        // Set selection outline width based on camera scale
        for mat_handle in query_mat.iter_mut() {
            if let Some(mat) = materials.get_mut(mat_handle) {
                mat.outline_width = 0.5 * ortho.scale;
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

    let mut batch_cmds = Vec::new();

    for evt in image_drop_event_reader.iter() {
        let mut width_ratio = 1.0;

        if let Ok(dim) = imagesize::size(evt.dropped_image_path.clone()) {
            // Check the maximum width and height before createing a texture
            // wgpu has a limit of 16384x16384
            if dim.width >= 16384 || dim.height >= 16384 {
                //TODO: Show error message
                continue;
            }

            // Make sure the image is 2D
            if dim.width <= 1 || dim.height <= 1 {
                continue;
            }

            width_ratio = dim.width as f32 / dim.height as f32;
        }

        let tex_handle = asset_server.load(evt.dropped_image_path.clone());
        let quad_handle = meshes.add(Mesh::from(shape::Quad::new(Vec2::new(
            QUAD_SIZE, QUAD_SIZE,
        ))));

        let mat_handle = materials.add(MaterialSeparateChannel {
            base_color_texture: Some(tex_handle),
            channel: 0,
            show_outline: 0,
            outline_color: Color::rgb_u8(118, 157, 240),
            outline_width: 1.0,
            quad_ratio: width_ratio,
        });

        let mut transform =
            Transform::from_xyz(evt.world_pos.x, evt.world_pos.y, img_count as f32 / 1000.0);

        if width_ratio != 1.0 {
            transform.scale = Vec3::new(width_ratio, 1.0, 1.0);
        }

        let bundle = (
            MaterialMeshBundle {
                mesh: quad_handle,
                material: mat_handle,
                transform,
                ..default()
            },
            PickableBundle::default(),
            RaycastPickTarget::default(),
            OnPointer::<DragStart>::target_remove::<Pickable>(),
            OnPointer::<Drag>::run_callback(drag_move_system),
            OnPointer::<DragEnd>::target_insert(Pickable),
            DropInImage,
        );

        batch_cmds.push(bundle);

        img_count += 1;
    }

    cmds.spawn_batch(batch_cmds);
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
    query: Query<(Entity, &PickSelection), With<DropInImage>>,
) {
    // press x to delete selected image
    if keyboard_input.pressed(KeyCode::X) && !keyboard_input.pressed(KeyCode::LShift) {
        for (entity, sel) in query.iter() {
            if sel.is_selected {
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

fn drag_move_system(
    In(event): In<ListenedEvent<Drag>>,
    keyboard_input: Res<Input<KeyCode>>,
    mouse_input: Res<Input<MouseButton>>,
    mut query: Query<(Entity, &PickSelection, &mut Transform), With<DropInImage>>,
    query_camera: Query<&Projection, With<CameraController>>,
) -> Bubble {
    let cam_projection = query_camera.single();

    if mouse_input.pressed(MouseButton::Left) && !keyboard_input.pressed(KeyCode::Space) {
        if let Projection::Orthographic(ortho) = cam_projection {
            let x = event.delta.x * DRAG_SPEED * ortho.scale;
            let y = event.delta.y * DRAG_SPEED * ortho.scale;

            let mut dragging_entity: Option<Entity> = None;

            if let Ok((entity, _selection, mut transform)) = query.get_mut(event.target) {
                dragging_entity = Some(entity);

                transform.translation.x += x;
                transform.translation.y += y;
            }

            for (_, selection, mut transform) in query
                .iter_mut()
                .filter(|(e, _, _)| dragging_entity.is_some() && dragging_entity.unwrap() != *e)
            {
                if selection.is_selected {
                    transform.translation.x += x;
                    transform.translation.y += y;
                }
            }
        }
    }

    Bubble::Up
}

fn rearrange_image_system(
    mut events: EventReader<RearrangeEvent>,
    mut query: Query<&mut Transform, With<DropInImage>>,
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

fn highlight_outline_system(
    mut selections: EventReader<PointerEvent<Select>>,
    mut deselections: EventReader<PointerEvent<Deselect>>,
    mut materials: ResMut<Assets<MaterialSeparateChannel>>,
    mut query: Query<&Handle<MaterialSeparateChannel>, With<PickSelection>>,
) {
    for selection in selections.iter() {
        if let Ok(mat_handle) = query.get_mut(selection.target) {
            if let Some(mat) = materials.get_mut(mat_handle) {
                mat.show_outline = 1;
            }
        }
    }

    for deselection in deselections.iter() {
        if let Ok(mat_handle) = query.get_mut(deselection.target) {
            if let Some(mat) = materials.get_mut(mat_handle) {
                mat.show_outline = 0;
            }
        }
    }
}
