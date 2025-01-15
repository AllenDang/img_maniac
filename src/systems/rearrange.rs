use std::path::PathBuf;

use bevy::prelude::*;
use taffy::{
    prelude::{auto, length, AlignContent, AlignItems, FlexDirection, FlexWrap, TaffyMaxContent},
    Size, Style, TaffyTree,
};

#[derive(Component)]
pub struct DropInImage {
    pub width: f32,
    pub height: f32,
    pub file_path: PathBuf,
}

#[derive(Event)]
pub struct EvtRearrange;

pub fn rearrange_system(
    mut evt_rearrange: EventReader<EvtRearrange>,
    mut q_image: Query<(&mut Transform, &DropInImage)>,
    q_windows: Query<&Window>,
    mut q_cam: Single<&mut Transform, (With<Camera2d>, Without<DropInImage>)>,
) {
    for _evt in evt_rearrange.read() {
        let mut tree: TaffyTree<()> = TaffyTree::new();

        let mut children = vec![];

        for (_, img) in q_image.iter() {
            let child_style = Style {
                size: Size {
                    width: length(img.width),
                    height: length(img.height),
                },
                ..default()
            };

            let child = tree.new_leaf(child_style).unwrap();
            children.push(child);
        }

        let win = q_windows.single();
        let win_size = win.size();

        let cam_scale = q_cam.scale.x;
        q_cam.translation = Vec3::ZERO;

        let root_node = tree
            .new_with_children(
                Style {
                    display: taffy::Display::Flex,
                    flex_direction: FlexDirection::Row,
                    flex_wrap: FlexWrap::Wrap,
                    justify_content: Some(AlignContent::Center),
                    align_items: Some(AlignItems::Center),
                    gap: Size {
                        width: length(16.),
                        height: length(16.),
                    },
                    size: Size {
                        width: length(win_size.x * cam_scale),
                        height: auto(),
                    },
                    ..default()
                },
                &children,
            )
            .unwrap();

        tree.compute_layout(root_node, Size::MAX_CONTENT).unwrap();

        let layout_size = tree.layout(root_node).unwrap().content_box_size();

        for (i, (mut trans, _)) in q_image.iter_mut().enumerate() {
            if let Some(&child) = children.get(i) {
                if let Ok(child_layout) = tree.layout(child) {
                    let child_size = child_layout.size;
                    trans.translation.x = child_layout.location.x + (child_size.width / 2.0)
                        - (layout_size.width / 2.0);
                    trans.translation.y = -(child_layout.location.y + (child_size.height / 2.0)
                        - (layout_size.height / 2.0));
                }
            }
        }
    }
}
