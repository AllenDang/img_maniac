use std::path::PathBuf;

use bevy::{
    input::mouse::{MouseMotion, MouseWheel},
    prelude::*,
    winit::WinitSettings,
};
use bevy_mod_picking::{
    InteractablePickingPlugin, PickableBundle, PickingCameraBundle, PickingEvent, PickingPlugin,
    Selection, SelectionEvent,
};
use mat_seperate_channel::MaterialSeperateChannel;

mod mat_seperate_channel;

struct ImageDropEvent {
    droped_image_path: PathBuf,
    world_pos: Vec2,
}

#[derive(Component)]
struct CameraController;

#[derive(Component)]
struct DropInImage;

#[derive(Component)]
struct Resized;

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
        .add_plugin(MaterialPlugin::<MaterialSeperateChannel>::default())
        .add_plugin(PickingPlugin)
        .add_plugin(InteractablePickingPlugin)
        .add_event::<ImageDropEvent>()
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
        ))
        .run()
}

fn startup_system(mut cmds: Commands) {
    // camera
    cmds.spawn((
        Camera3dBundle {
            transform: Transform::from_xyz(0.0, 0.0, 8.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        CameraController,
        PickingCameraBundle::default(),
    ));
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

            if path_buf.is_file() {
                let ext = path_buf.extension().unwrap().to_str().unwrap();
                if ["bmp", "dds", "exr", "jpeg", "jpg", "tga", "png"].contains(&ext) {
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
                        droped_image_path: path_buf.clone(),
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
}

fn change_cursor_system(
    keyboard_input: Res<Input<KeyCode>>,
    mouse_input: Res<Input<MouseButton>>,
    mut windows: Query<&mut Window>,
) {
    let mut window = windows.single_mut();

    if keyboard_input.pressed(KeyCode::Space) && !mouse_input.pressed(MouseButton::Left) {
        if window.cursor.icon != CursorIcon::Grab {
            window.cursor.icon = CursorIcon::Grab;
        }
    } else if keyboard_input.pressed(KeyCode::Space) && mouse_input.pressed(MouseButton::Left) {
        if window.cursor.icon != CursorIcon::Grabbing {
            window.cursor.icon = CursorIcon::Grabbing;
        }
    } else if window.cursor.icon != CursorIcon::Default {
        window.cursor.icon = CursorIcon::Default;
    }
}

fn camera_control_system(
    keyboard_input: Res<Input<KeyCode>>,
    mouse_input: Res<Input<MouseButton>>,
    mut mouse_motion_events: EventReader<MouseMotion>,
    mut mouse_wheel_events: EventReader<MouseWheel>,
    mut query: Query<&mut Transform, With<CameraController>>,
) {
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

    // Handle mouse wheel input to translate camera's z position
    for event in mouse_wheel_events.iter() {
        for mut transform in query.iter_mut() {
            let new_z = (transform.translation.z - event.y * 0.1).max(1.0).min(20.0);

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
    mut materials: ResMut<Assets<MaterialSeperateChannel>>,
    imgs: Query<&DropInImage>,
    asset_server: Res<AssetServer>,
) {
    let mut img_count = imgs.iter().count() + 1;

    for evt in image_drop_event_reader.iter() {
        let tex_handle = asset_server.load(evt.droped_image_path.clone());
        let quad_handle = meshes.add(Mesh::from(shape::Quad::new(Vec2::new(3.0, 3.0))));

        let mat_handle = materials.add(MaterialSeperateChannel {
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
        (Entity, &Handle<MaterialSeperateChannel>, &mut Transform),
        (With<DropInImage>, Without<Resized>),
    >,
    materials: Res<Assets<MaterialSeperateChannel>>,
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
    mut materials: ResMut<Assets<MaterialSeperateChannel>>,
    mut query: Query<&Handle<MaterialSeperateChannel>, With<DropInImage>>,
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
    mut materials: ResMut<Assets<MaterialSeperateChannel>>,
    mut query_mat: Query<&Handle<MaterialSeperateChannel>>,
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
