use crate::Pixelate;
use bevy::prelude::*;
use bevy::scene::SceneInstance;
use bevy::utils::{HashMap, HashSet};

#[derive(Debug, Resource, Reflect, Default, Deref, DerefMut)]
#[reflect(Resource)]
pub(crate) struct ToPixelate(HashSet<Entity>);

pub(crate) fn mark_for_pixelation(
    mut pixelate_query: Query<Entity, Added<Pixelate>>,
    mut to_pixelate: ResMut<ToPixelate>,
) {
    for entity in &mut pixelate_query {
        debug!("Adding entity to pixelation queue");
        to_pixelate.0.insert(entity);
    }
}

#[derive(Debug, Default, Deref, DerefMut)]
pub(crate) struct PixelationTargetReadyEvent(HashMap<Entity, PixelationTarget>);

#[derive(Debug, Clone)]
pub(crate) struct PixelationTarget {
    pub(crate) mesh_handle: Handle<Mesh>,
    pub(crate) kind: PixelationTargetKind,
}

#[derive(Debug, Clone)]
pub(crate) enum PixelationTargetKind {
    Mesh,
    Scene,
}

pub(crate) fn get_ready_pixelation_targets(
    mut to_pixelate: ResMut<ToPixelate>,
    pixelate_query: Query<
        (
            Option<&Handle<Mesh>>,
            Option<&Handle<Scene>>,
            Option<&SceneInstance>,
        ),
        With<Pixelate>,
    >,
    mesh_handles: Query<&Handle<Mesh>>,
    meshes: Res<Assets<Mesh>>,
    scene_spawner: Res<SceneSpawner>,
    mut pixelation_target_ready_event: EventWriter<PixelationTargetReadyEvent>,
) {
    let mut pixelation_targets = HashMap::new();
    for &entity in to_pixelate.iter() {
        if let Ok((mesh_handle, scene_handle, scene_instance)) = pixelate_query.get(entity) {
            if scene_handle.is_some() {
                debug!("Pixelating a scene; waiting for it to load...");
                if let Some(scene_instance) = scene_instance {
                    debug!("The scene is loaded, waiting for it to be ready...");
                    let scene_instance = **scene_instance;
                    if scene_spawner.instance_is_ready(scene_instance) {
                        let mesh_handle = scene_spawner
                            .iter_instance_entities(scene_instance)
                            .find_map(|entity| mesh_handles.get(entity).ok())
                            .unwrap();

                        debug!("The scene is ready!");
                        pixelation_targets.insert(
                            entity,
                            PixelationTarget {
                                mesh_handle: mesh_handle.clone(),
                                kind: PixelationTargetKind::Scene,
                            },
                        );
                    }
                }
            } else if let Some(mesh_handle) = mesh_handle {
                debug!("Pixelating a mesh; waiting for it to load...");
                if meshes.contains(mesh_handle) {
                    debug!("The mesh is loaded!");
                    pixelation_targets.insert(
                        entity,
                        PixelationTarget {
                            mesh_handle: mesh_handle.clone(),
                            kind: PixelationTargetKind::Mesh,
                        },
                    );
                }
            } else {
                panic!("The Pixelate component can only be added to entities with a Mesh or a Scene, but found neither.");
            }
        }
    }
    let ready = pixelation_targets.keys().copied().collect();
    to_pixelate.0 = to_pixelate.difference(&ready).copied().collect();
    pixelation_target_ready_event.send(PixelationTargetReadyEvent(pixelation_targets));
}
