use bevy::prelude::*;
use rand::RngExt;

use crate::SceneStats;

#[derive(Resource)]
pub struct LowDensityBuildings {
    meshes: Vec<Handle<Mesh>>,
    materials: Vec<Handle<StandardMaterial>>,
}

impl LowDensityBuildings {
    fn random_building<R: RngExt>(
        &self,
        rng: &mut R,
    ) -> (Mesh3d, MeshMaterial3d<StandardMaterial>) {
        let mesh = self.meshes[rng.random_range(0..self.meshes.len())].clone();
        let material = self.materials[rng.random_range(0..self.materials.len())].clone();
        (Mesh3d(mesh), MeshMaterial3d(material))
    }
}

pub fn load_low_density_buildings(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let meshes = ["b", "c", "d", "e", "f", "g", "h", "i", "k", "l", "o", "u"]
        .iter()
        .map(|t| {
            asset_server.load(
                GltfAssetLabel::Primitive {
                    mesh: 0,
                    primitive: 0,
                }
                .from_asset(format!("low_density/building-type-{t}.glb")),
            )
        })
        .collect::<Vec<_>>();
    let materials = ["colormap", "variation-a", "variation-b", "variation-c"]
        .iter()
        .map(|variation| {
            materials.add(StandardMaterial {
                base_color_texture: Some(
                    asset_server.load(format!("low_density/Textures/{variation}.png")),
                ),
                ..Default::default()
            })
        })
        .collect::<Vec<_>>();
    commands.insert_resource(LowDensityBuildings { meshes, materials });
}

pub fn spawn_low_density<R: RngExt>(
    commands: &mut Commands,
    stats: &mut SceneStats,
    mut rng: &mut R,
    offset: Vec3,
    low_density_buildings: &LowDensityBuildings,
    tree_small: Handle<Scene>,
    fence: Handle<Scene>,
) {
    for z in 0..=8 {
        commands.spawn((
            SceneRoot(tree_small.clone()),
            Transform::from_translation(Vec3::new(0.75, 0.0, 0.75 + z as f32 * 0.3) + offset),
        ));
        stats.trees += 1;
        commands.spawn((
            SceneRoot(tree_small.clone()),
            Transform::from_translation(Vec3::new(4.75, 0.0, 0.75 + z as f32 * 0.3) + offset),
        ));
        stats.trees += 1;
    }
    for i in 0..=6 {
        commands.spawn((
            SceneRoot(fence.clone()),
            Transform::from_translation(Vec3::new(2.75, 0.0, 0.75 + i as f32 * 0.4) + offset)
                .with_rotation(Quat::from_axis_angle(Vec3::Y, std::f32::consts::FRAC_PI_2)),
        ));
    }
    for x in 1..=2 {
        let x_factor = 1.8;
        commands.spawn((
            low_density_buildings.random_building(&mut rng),
            Transform::from_translation(Vec3::new(x as f32 * x_factor, 0.0, 1.25) + offset),
        ));
        stats.low_density_buildings += 1;
        commands.spawn((
            low_density_buildings.random_building(&mut rng),
            Transform::from_translation(Vec3::new(x as f32 * x_factor, 0.0, 2.75) + offset)
                .with_rotation(Quat::from_axis_angle(Vec3::Y, std::f32::consts::PI)),
        ));
        stats.low_density_buildings += 1;
    }
}
