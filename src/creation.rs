use crate::ready_checks::PixelationTargetReadyEvent;
use crate::util::{get_max_radius, get_pixelation_render_layer};
use crate::{Canvas, Pixelate, PixelationCamera};
use bevy::{
    core_pipeline::clear_color::ClearColorConfig,
    prelude::*,
    render::{
        camera::RenderTarget,
        primitives::Aabb,
        render_resource::{
            Extent3d, TextureDescriptor, TextureDimension, TextureFormat, TextureUsages,
        },
    },
};
use std::f32::consts::PI;

#[derive(Debug, Resource, Clone, Default)]
pub(crate) struct Ordering {
    pub(crate) last_order: isize,
}
impl Ordering {
    pub(crate) fn next(&mut self) -> isize {
        // render before the "main pass" camera
        self.last_order -= 1;
        self.last_order
    }
}

pub(crate) fn add_pixelation(
    mut commands: Commands,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut images: ResMut<Assets<Image>>,
    pixelate_query: Query<&Pixelate>,
    mut pixelation_target_ready_reader: EventReader<PixelationTargetReadyEvent>,
    mut ordering: ResMut<Ordering>,
) {
    // This specifies the layer used for the first pass, which will be attached to the first pass camera and cube.
    let first_pass_layer = get_pixelation_render_layer();
    for event in pixelation_target_ready_reader.iter() {
        for (&entity, target) in event.iter() {
            debug!("Spawning canvas");
            let mesh = meshes.get(&target.mesh_handle).unwrap();
            let aabb = mesh.compute_aabb().unwrap();
            let plane_handle = meshes.add(create_canvas_mesh(&aabb));
            let pixelate = pixelate_query.get(entity).unwrap();
            let image = create_image(*pixelate);
            let image_handle = images.add(image);
            commands
                .entity(entity)
                .insert((first_pass_layer, aabb.clone()));
            commands.spawn((
                Name::new("Pixelation Camera"),
                Camera3dBundle {
                    camera_3d: Camera3d {
                        clear_color: ClearColorConfig::Custom(Color::NONE),
                        ..default()
                    },
                    camera: Camera {
                        order: ordering.next(),
                        target: RenderTarget::Image(image_handle.clone()),
                        ..default()
                    },
                    ..default()
                },
                PixelationCamera { target: entity },
                first_pass_layer,
            ));

            commands
                .spawn((
                    Name::new("Canvas"),
                    Canvas { target: entity },
                    SpatialBundle::default(),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Name::new("Canvas Mesh"),
                        PbrBundle {
                            mesh: plane_handle,
                            material: materials.add(StandardMaterial {
                                base_color_texture: Some(image_handle.clone()),
                                unlit: true,
                                alpha_mode: AlphaMode::Blend,
                                ..default()
                            }),
                            transform: Transform::from_rotation(Quat::from_rotation_y(PI)),
                            ..default()
                        },
                    ));
                });
        }
    }
}

fn create_canvas_mesh(aabb: &Aabb) -> Mesh {
    let radius = get_max_radius(aabb);
    let size = Vec2::splat(radius * 2.);
    Mesh::from(shape::Quad { size, flip: false })
}

fn create_image(pixelate: Pixelate) -> Image {
    let size = Extent3d {
        width: pixelate.horizontal_pixels,
        height: pixelate.vertical_pixels,
        ..default()
    };
    // This is the texture that will be rendered to.
    let mut image = Image {
        texture_descriptor: TextureDescriptor {
            label: Some("pixelated render to texture"),
            size,
            dimension: TextureDimension::D2,
            format: TextureFormat::Bgra8UnormSrgb,
            mip_level_count: 1,
            sample_count: 1,
            usage: TextureUsages::TEXTURE_BINDING
                | TextureUsages::COPY_DST
                | TextureUsages::RENDER_ATTACHMENT,
            view_formats: &[],
        },
        ..default()
    };
    // fill image.data with zeroes
    image.resize(size);
    image
}
