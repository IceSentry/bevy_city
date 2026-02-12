use bevy::prelude::*;
use rand::RngExt;

use crate::SceneStats;

#[derive(Resource)]
pub struct SkyscraperBuildings {
    meshes: Vec<Handle<Mesh>>,
    materials: Vec<Handle<StandardMaterial>>,
}

impl SkyscraperBuildings {
    pub fn random_building<R: RngExt>(
        &self,
        rng: &mut R,
    ) -> (Mesh3d, MeshMaterial3d<StandardMaterial>) {
        let mesh = self.meshes[rng.random_range(0..self.meshes.len())].clone();
        let material = self.materials[rng.random_range(0..self.materials.len())].clone();
        (Mesh3d(mesh), MeshMaterial3d(material))
    }
}

pub fn load_skyscrapers(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mut meshes = ["a", "b", "c", "d", "e"]
        .iter()
        .map(|t| {
            asset_server.load(
                GltfAssetLabel::Primitive {
                    mesh: 0,
                    primitive: 0,
                }
                .from_asset(format!("high_density/building-skyscraper-{t}.glb")),
            )
        })
        .collect::<Vec<_>>();
    meshes.push(
        asset_server.load(
            GltfAssetLabel::Primitive {
                mesh: 0,
                primitive: 0,
            }
            .from_asset("high_density/building-m.glb"),
        ),
    );
    meshes.push(
        asset_server.load(
            GltfAssetLabel::Primitive {
                mesh: 0,
                primitive: 0,
            }
            .from_asset("high_density/building-l.glb"),
        ),
    );
    let materials = ["colormap", "variation-a", "variation-b"]
        .iter()
        .map(|variation| {
            materials.add(StandardMaterial {
                base_color_texture: Some(
                    asset_server.load(format!("high_density/Textures/{variation}.png")),
                ),
                ..Default::default()
            })
        })
        .collect::<Vec<_>>();
    commands.insert_resource(SkyscraperBuildings { meshes, materials });
}

pub fn spawn_high_density<R: RngExt>(
    commands: &mut Commands,
    stats: &mut SceneStats,
    skyscrapers: &SkyscraperBuildings,
    mut rng: &mut R,
    offset: Vec3,
) {
    for x in 0..3 {
        commands.spawn((
            skyscrapers.random_building(&mut rng),
            Transform::from_translation(Vec3::new(1.25 + x as f32 * 1.5, 0.0, 1.25) + offset),
        ));
        stats.skyscrapers += 1;
        commands.spawn((
            skyscrapers.random_building(&mut rng),
            Transform::from_translation(Vec3::new(1.25 + x as f32 * 1.5, 0.0, 2.75) + offset)
                .with_rotation(Quat::from_axis_angle(Vec3::Y, std::f32::consts::PI)),
        ));
        stats.skyscrapers += 1;
    }
}
