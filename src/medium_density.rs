use bevy::prelude::*;
use rand::RngExt;

use crate::SceneStats;

#[derive(Resource)]
pub struct MediumDensityBuildings {
    meshes: Vec<Handle<Mesh>>,
    materials: Vec<Handle<StandardMaterial>>,
}

impl MediumDensityBuildings {
    pub fn random_building<R: RngExt>(
        &self,
        rng: &mut R,
    ) -> (Mesh3d, MeshMaterial3d<StandardMaterial>) {
        let mesh = self.meshes[rng.random_range(0..self.meshes.len())].clone();
        let material = self.materials[rng.random_range(0..self.materials.len())].clone();
        (Mesh3d(mesh), MeshMaterial3d(material))
    }
}

pub fn load_medium_density_buildings(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let meshes = ["a", "b", "c", "d", "f", "g", "h"]
        .iter()
        .map(|t| {
            asset_server.load(
                GltfAssetLabel::Primitive {
                    mesh: 0,
                    primitive: 0,
                }
                .from_asset(format!("kenney_city_commercial/building-{t}.glb")),
            )
        })
        .collect::<Vec<_>>();

    let materials = ["colormap", "variation-a", "variation-b"]
        .iter()
        .map(|variation| {
            materials.add(StandardMaterial {
                base_color_texture: Some(
                    asset_server.load(format!("kenney_city_commercial/Textures/{variation}.png")),
                ),
                ..Default::default()
            })
        })
        .collect::<Vec<_>>();
    commands.insert_resource(MediumDensityBuildings { meshes, materials });
}

pub fn spawn_medium_density<R: RngExt>(
    commands: &mut Commands,
    stats: &mut SceneStats,
    mut rng: &mut R,
    offset: Vec3,
    medium_density_buildings: &MediumDensityBuildings,
    trees: &[Handle<Scene>],
    path_stones_long: Handle<Scene>,
) {
    // Use the same tree for the entire alley
    let tree = trees[rng.random_range(0..2)].clone();

    // Squish things a little so we can spawn more buildings
    let x_factor = 0.9;
    for x in 1..=5 {
        commands.spawn((
            medium_density_buildings.random_building(&mut rng),
            Transform::from_translation(Vec3::new(x as f32 * x_factor, 0.0, 1.0) + offset),
        ));
        stats.medium_density_buildings += 1;
        for tree_x in 0..=1 {
            let tree_x = tree_x as f32 * 0.5;
            if x == 5 && tree_x == 0.5 {
                break;
            }
            commands.spawn((
                SceneRoot(tree.clone()),
                Transform::from_translation(
                    Vec3::new(tree_x + x as f32 * x_factor, 0.0, 1.75) + offset,
                ),
            ));
            stats.trees += 1;
            commands.spawn((
                SceneRoot(tree.clone()),
                Transform::from_translation(
                    Vec3::new(tree_x + x as f32 * x_factor, 0.0, 2.25) + offset,
                ),
            ));
            stats.trees += 1;
        }
        commands.spawn((
            medium_density_buildings.random_building(&mut rng),
            Transform::from_translation(Vec3::new(x as f32 * x_factor, 0.0, 3.0) + offset)
                .with_rotation(Quat::from_axis_angle(Vec3::Y, std::f32::consts::PI)),
        ));
        stats.medium_density_buildings += 1;
    }

    // path
    for x in 0..=10 {
        commands.spawn((
            SceneRoot(path_stones_long.clone()),
            Transform::from_translation(Vec3::new(0.75 + (x as f32 * 0.4), 0.02, 2.0) + offset)
                .with_scale(Vec3::new(1.0, 2.0, 1.0))
                .with_rotation(Quat::from_axis_angle(Vec3::Y, std::f32::consts::FRAC_PI_2)),
        ));
    }
}
