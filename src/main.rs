use std::f32::consts;

use bevy::anti_alias::taa::TemporalAntiAliasing;
use bevy::camera::Exposure;
use bevy::camera_controller::free_camera::{FreeCamera, FreeCameraPlugin};
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::light::{AtmosphereEnvironmentMapLight, VolumetricFog, VolumetricLight};
use bevy::pbr::{Atmosphere, AtmosphereSettings, ScatteringMedium, ScreenSpaceReflections};
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use rand::rngs::SmallRng;
use rand::{RngExt, SeedableRng};

fn main() {
    App::new()
        .add_plugins((DefaultPlugins, FreeCameraPlugin))
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(GlobalAmbientLight::NONE)
        .add_systems(Startup, (setup_camera, setup_city))
        .run();
}

fn setup_camera(mut commands: Commands, mut scattering_mediums: ResMut<Assets<ScatteringMedium>>) {
    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 5.0, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        FreeCamera::default(),
        Atmosphere::earthlike(scattering_mediums.add(ScatteringMedium::default())),
        AtmosphereSettings::default(),
        // The directional light illuminance used in this scene is
        // quite bright, so raising the exposure compensation helps
        // bring the scene to a nicer brightness range.
        Exposure { ev100: 13.0 },
        // Bloom gives the sun a much more natural look.
        Bloom::NATURAL,
        // Enables the atmosphere to drive reflections and ambient lighting (IBL) for this view
        AtmosphereEnvironmentMapLight::default(),
        VolumetricFog {
            ambient_intensity: 0.0,
            ..default()
        },
        // Msaa::Off,
        // TemporalAntiAliasing::default(),
        ScreenSpaceReflections::default(),
    ));

    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,

            illuminance: light_consts::lux::RAW_SUNLIGHT,
            ..default()
        },
        Transform::from_xyz(1.0, 2.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        VolumetricLight,
    ));
}

#[allow(clippy::collapsible_else_if)]
fn setup_city(mut commands: Commands, asset_server: Res<AssetServer>) {
    let crossroad: Handle<Scene> = asset_server
        .load(GltfAssetLabel::Scene(0).from_asset("kenney_roads/road-crossroad-path.glb"));
    let straight: Handle<Scene> =
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("kenney_roads/road-straight.glb"));
    let straight_half: Handle<Scene> = asset_server
        .load(GltfAssetLabel::Scene(0).from_asset("kenney_roads/road-straight-half.glb"));
    let tile: Handle<Scene> =
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("kenney_roads/tile-low.glb"));

    // 1x1 buildings
    let buildings: Vec<Handle<Scene>> = vec![
        asset_server
            .load(GltfAssetLabel::Scene(0).from_asset("kenney_city_commercial/building-a.glb")),
        asset_server
            .load(GltfAssetLabel::Scene(0).from_asset("kenney_city_commercial/building-b.glb")),
        asset_server
            .load(GltfAssetLabel::Scene(0).from_asset("kenney_city_commercial/building-c.glb")),
        asset_server
            .load(GltfAssetLabel::Scene(0).from_asset("kenney_city_commercial/building-d.glb")),
        asset_server
            .load(GltfAssetLabel::Scene(0).from_asset("kenney_city_commercial/building-f.glb")),
        asset_server
            .load(GltfAssetLabel::Scene(0).from_asset("kenney_city_commercial/building-g.glb")),
        asset_server
            .load(GltfAssetLabel::Scene(0).from_asset("kenney_city_commercial/building-h.glb")),
    ];
    let skyscrapers: Vec<Handle<Scene>> = vec![
        asset_server.load(
            GltfAssetLabel::Scene(0).from_asset("kenney_city_commercial/building-skyscraper-a.glb"),
        ),
        asset_server.load(
            GltfAssetLabel::Scene(0).from_asset("kenney_city_commercial/building-skyscraper-b.glb"),
        ),
        asset_server.load(
            GltfAssetLabel::Scene(0).from_asset("kenney_city_commercial/building-skyscraper-c.glb"),
        ),
        asset_server.load(
            GltfAssetLabel::Scene(0).from_asset("kenney_city_commercial/building-skyscraper-d.glb"),
        ),
        asset_server.load(
            GltfAssetLabel::Scene(0).from_asset("kenney_city_commercial/building-skyscraper-e.glb"),
        ),
    ];

    let mut rng = SmallRng::seed_from_u64(42);

    let mut spawn_city_block = |offset: Vec3| {
        commands.spawn((
            SceneRoot(crossroad.clone()),
            Transform::from_translation(offset),
        ));

        // X roads
        commands.spawn((
            SceneRoot(straight.clone()),
            Transform::from_translation(Vec3::new(1.0, 0.0, 0.0) + offset),
        ));
        commands.spawn((
            SceneRoot(straight.clone()),
            Transform::from_translation(Vec3::new(2.0, 0.0, 0.0) + offset),
        ));
        commands.spawn((
            SceneRoot(straight_half.clone()),
            Transform::from_translation(Vec3::new(2.75, 0.0, 0.0) + offset),
        ));
        commands.spawn((
            SceneRoot(straight.clone()),
            Transform::from_translation(Vec3::new(3.5, 0.0, 0.0) + offset),
        ));
        commands.spawn((
            SceneRoot(straight.clone()),
            Transform::from_translation(Vec3::new(4.5, 0.0, 0.0) + offset),
        ));

        // Z roads
        commands.spawn((
            SceneRoot(straight.clone()),
            Transform::from_translation(Vec3::new(0.0, 0.0, 1.0) + offset)
                .with_rotation(Quat::from_axis_angle(Vec3::Y, std::f32::consts::FRAC_PI_2)),
        ));
        commands.spawn((
            SceneRoot(straight.clone()),
            Transform::from_translation(Vec3::new(0.0, 0.0, 2.0) + offset)
                .with_rotation(Quat::from_axis_angle(Vec3::Y, std::f32::consts::FRAC_PI_2)),
        ));
        commands.spawn((
            SceneRoot(straight.clone()),
            Transform::from_translation(Vec3::new(0.0, 0.0, 3.0) + offset)
                .with_rotation(Quat::from_axis_angle(Vec3::Y, std::f32::consts::FRAC_PI_2)),
        ));

        let tile_scale = Vec3::new(4.5, 1.0, 3.0);

        commands.spawn((
            SceneRoot(tile.clone()),
            Transform::from_translation(Vec3::new(0.5, -0.5005, 0.5) + tile_scale / 2.0 + offset)
                .with_scale(tile_scale),
        ));

        let use_skyscrapers = rng.random_bool(0.25);
        if use_skyscrapers {
            for x in 0..3 {
                commands.spawn((
                    SceneRoot(skyscrapers[rng.random_range(0..skyscrapers.len())].clone()),
                    Transform::from_translation(
                        Vec3::new(1.25 + x as f32 * 1.5, 0.0, 1.25) + offset,
                    ),
                ));
                commands.spawn((
                    SceneRoot(skyscrapers[rng.random_range(0..skyscrapers.len())].clone()),
                    Transform::from_translation(
                        Vec3::new(1.25 + x as f32 * 1.5, 0.0, 2.75) + offset,
                    )
                    .with_rotation(Quat::from_axis_angle(Vec3::Y, std::f32::consts::PI)),
                ));
            }
        } else {
            // Squish things a little so we can spawn more buildings
            let x_factor = 0.9;
            for x in 1..=5 {
                commands.spawn((
                    SceneRoot(buildings[rng.random_range(0..buildings.len())].clone()),
                    Transform::from_translation(Vec3::new(x as f32 * x_factor, 0.0, 1.0) + offset),
                ));
                commands.spawn((
                    SceneRoot(buildings[rng.random_range(0..buildings.len())].clone()),
                    Transform::from_translation(Vec3::new(x as f32 * x_factor, 0.0, 3.0) + offset)
                        .with_rotation(Quat::from_axis_angle(Vec3::Y, std::f32::consts::PI)),
                ));
            }
        }
    };

    let size = 20;
    for x in -size..=size {
        for z in -size..=size {
            spawn_city_block(Vec3::new(x as f32 * 5.5, 0.0, z as f32 * 4.0));
        }
    }
}
