use bevy::{
    asset::Assets,
    image::{Image, ImageSampler},
    input::{
        mouse::{MouseMotion, MouseWheel},
        ButtonInput,
    },
    math::{FloatExt, Vec2, Vec3},
    prelude::{
        Camera, Camera2d, Component, EventReader, GlobalTransform, MouseButton, Query, Res, ResMut,
        Transform, With,
    },
    sprite::MeshMaterial2d,
    time::Time,
    window::Window,
};

use crate::shader::mat_separate_channel::MaterialSeparateChannel;

use super::rearrange::DropInImage;

#[derive(Component)]
pub struct CamStatus {
    pub target_scale: f32,
    pub current_scale: f32,
    pub enable_pixel_perfect: bool,
}

#[allow(clippy::too_many_arguments)]
pub fn cam_zoom_system(
    mut cam: Query<(&mut Transform, &mut CamStatus), With<Camera2d>>,
    mut evt_mouse_wheel: EventReader<MouseWheel>,
    time: Res<Time>,
    window: Query<&Window>,
    q_camera: Query<(&Camera, &GlobalTransform), With<Camera2d>>,
    mut materials: ResMut<Assets<MaterialSeparateChannel>>,
    mut q_mat: Query<&MeshMaterial2d<MaterialSeparateChannel>, With<DropInImage>>,
    mut images: ResMut<Assets<Image>>,
) {
    const ZOOM_IN_SPEED: f32 = 0.05;
    const ZOOM_OUT_SPEED: f32 = 0.1;
    const MIN_ZOOM: f32 = 0.05;
    const MAX_ZOOM: f32 = 20.0;
    const SMOOTH_SPEED: f32 = 12.0;
    const MAX_DELTA_TIME: f32 = 1.0 / 60.0;

    let (mut transform, mut status) = cam.single_mut();
    let window = window.single();
    let (camera, camera_transform) = q_camera.single();

    let scroll = evt_mouse_wheel.read().map(|ev| ev.y).sum::<f32>();

    if scroll == 0.0 && (status.target_scale - status.current_scale).abs() < 0.001 {
        return;
    }

    if let Some(cursor_pos) = window.cursor_position() {
        if let Ok(cursor_world_pos) = camera.viewport_to_world_2d(camera_transform, cursor_pos) {
            let old_scale = status.current_scale;

            if scroll != 0.0 {
                // Calculate zoom factor
                let zoom_speed = if scroll > 0.0 {
                    ZOOM_OUT_SPEED
                } else {
                    ZOOM_IN_SPEED
                };
                let zoom_factor = (-scroll * zoom_speed).exp();
                status.target_scale = (status.target_scale * zoom_factor).clamp(MIN_ZOOM, MAX_ZOOM);
            }

            // Smoothly interpolate to target scale
            let delta_time = time.delta_secs().min(MAX_DELTA_TIME);
            let new_scale = status
                .current_scale
                .lerp(status.target_scale, SMOOTH_SPEED * delta_time);

            if (new_scale - old_scale).abs() > 0.0001 {
                // Get current camera position in world space
                let camera_pos = transform.translation.truncate();

                // Calculate the vector from camera to cursor in world space
                let camera_to_cursor = cursor_world_pos - camera_pos;

                // Calculate how this vector should change with the new scale
                let scale_change = new_scale / old_scale;
                let position_compensation = camera_to_cursor * (1.0 - scale_change);

                // Apply new scale
                status.current_scale = new_scale;
                transform.scale = Vec3::splat(new_scale);

                // Apply position compensation to keep cursor point stationary
                transform.translation += position_compensation.extend(0.0);

                // Dynamic change image sampler
                let mut change_image_sampler = |sampler: ImageSampler| {
                    for mat_handle in q_mat.iter_mut() {
                        if let Some(mat) = materials.get_mut(mat_handle) {
                            if let Some(tex) = &mat.base_color_texture {
                                if let Some(img) = images.get_mut(tex.id()) {
                                    img.sampler = sampler.to_owned();
                                }
                            }
                        }
                    }
                };

                if new_scale <= 0.1 {
                    if !status.enable_pixel_perfect {
                        change_image_sampler(ImageSampler::nearest());
                        status.enable_pixel_perfect = true;
                    }
                } else if status.enable_pixel_perfect {
                    change_image_sampler(ImageSampler::linear());
                    status.enable_pixel_perfect = false;
                }
            }
        }
    }
}

pub fn cam_move_system(
    mut cam: Query<&mut Transform, With<Camera2d>>,
    btn: Res<ButtonInput<MouseButton>>,
    mut motion_evr: EventReader<MouseMotion>,
) {
    const MOVE_SPEED: f32 = 1.0; // Adjust this value to change movement speed

    if !btn.pressed(MouseButton::Middle) {
        motion_evr.clear();
        return;
    }

    let mut transform = cam.single_mut();

    let mouse_delta = motion_evr.read().fold(Vec2::ZERO, |acc, ev| acc + ev.delta);

    if mouse_delta == Vec2::ZERO {
        return;
    }

    let camera_scale = transform.scale.x;
    let translation = -Vec2::new(
        mouse_delta.x * camera_scale * MOVE_SPEED,
        -mouse_delta.y * camera_scale * MOVE_SPEED,
    );

    transform.translation.x += translation.x;
    transform.translation.y += translation.y;
}
