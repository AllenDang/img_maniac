use bevy::{asset::AssetServer, prelude::*, window::FileDragAndDrop};
use std::{
    fs,
    path::{Path, PathBuf},
};

use crate::{shader::mat_separate_channel::MaterialSeparateChannel, systems};

use super::rearrange::EvtRearrange;

fn get_files_recursive<P: AsRef<Path>>(
    path: P,
) -> impl Iterator<Item = std::io::Result<fs::DirEntry>> {
    fs::read_dir(path)
        .expect("Directory not found")
        .flat_map(|res| {
            let dir_entry = res.expect("Error reading directory");
            if dir_entry.path().is_dir() {
                Box::new(get_files_recursive(dir_entry.path())) as Box<dyn Iterator<Item = _>>
            } else {
                Box::new(std::iter::once(Ok(dir_entry))) as Box<dyn Iterator<Item = _>>
            }
        })
}

pub fn is_supported_format(pb: &Path) -> bool {
    if pb.is_file() {
        if let Some(ext) = pb.extension() {
            if let Some(ext_str) = ext.to_str() {
                match ext_str.to_lowercase().as_str() {
                    "jpg" | "jpeg" | "png" | "pnm" | "bmp" | "dds" | "exr" | "tga" | "tiff"
                    | "ico" | "qoi" | "hdr" | "webp" | "ff" => return true,
                    _ => return false,
                }
            }
        }
    }

    false
}

pub fn file_drop_system(
    mut evt_file_drag_and_drop: EventReader<FileDragAndDrop>,
    mut evt_rearrage: EventWriter<EvtRearrange>,
    mut cmds: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<MaterialSeparateChannel>>,
    asset_server: Res<AssetServer>,
) {
    const MAX_SIZE: usize = 16384;
    const FIXED_SIZE: f32 = 600.;

    for event in evt_file_drag_and_drop.read() {
        let mut dropped_files: Vec<PathBuf> = vec![];
        if let FileDragAndDrop::DroppedFile {
            window: _,
            path_buf,
        } = event
        {
            if path_buf.is_file() {
                if is_supported_format(path_buf) {
                    dropped_files.push(path_buf.to_owned());
                }
            } else {
                get_files_recursive(path_buf).for_each(|entry| {
                    if let Ok(e) = entry {
                        let path_buf = &e.path();
                        if is_supported_format(path_buf) {
                            dropped_files.push(path_buf.to_owned());
                        }
                    }
                });
            }
        }

        if dropped_files.is_empty() {
            return;
        }

        let mut batch_cmds = vec![];

        for (i, img_file_path) in dropped_files.iter().enumerate() {
            if let Ok(img_size) = imagesize::size(img_file_path) {
                let width = img_size.width;
                let height = img_size.height;

                // ignore large image
                if width >= MAX_SIZE || height >= MAX_SIZE {
                    continue;
                }

                // make sure the image is valid
                if width <= 1 || height <= 1 {
                    continue;
                }

                let scale_ratio = FIXED_SIZE / width as f32;

                let mut trans = Transform::from_scale(Vec3::new(scale_ratio, scale_ratio, 1.));
                trans.translation.z = i as f32;

                let rect = Rectangle::from_size(Vec2::new(
                    width as f32 * scale_ratio,
                    height as f32 * scale_ratio,
                ));

                batch_cmds.push((
                    Mesh2d(meshes.add(rect)),
                    MeshMaterial2d(materials.add(MaterialSeparateChannel {
                        channel: 0,
                        show_outline: 0,
                        outline_color: LinearRgba::rgb(118., 157., 240.),
                        outline_width: 1.0,
                        quad_ratio: scale_ratio,
                        base_color_texture: Some(asset_server.load(img_file_path.as_path())),
                    })),
                    systems::rearrange::DropInImage {
                        width: rect.size().x,
                        height: rect.size().y,
                    },
                ));
            }
        }

        if !batch_cmds.is_empty() {
            cmds.spawn_batch(batch_cmds);
            evt_rearrage.send(EvtRearrange);
        }
    }
}
