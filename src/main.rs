use std::f32::consts;

use bevy::anti_alias::taa::TemporalAntiAliasing;
use bevy::camera::Exposure;
use bevy::camera_controller::free_camera::{FreeCamera, FreeCameraPlugin, FreeCameraState};
use bevy::core_pipeline::tonemapping::Tonemapping;
use bevy::light::{
    AtmosphereEnvironmentMapLight, CascadeShadowConfigBuilder, FogVolume, VolumetricFog,
    VolumetricLight,
};
use bevy::pbr::{Atmosphere, AtmosphereSettings, ScatteringMedium, ScreenSpaceReflections};
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;

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
        Transform::from_xyz(-2.5, 4.5, 9.0).looking_at(Vec3::ZERO, Vec3::Y),
        FreeCamera::default(),
        // Earthlike atmosphere
        Atmosphere::earthlike(scattering_mediums.add(ScatteringMedium::default())),
        // Can be adjusted to change the scene scale and rendering quality
        AtmosphereSettings::default(),
        // The directional light illuminance used in this scene
        // (the one recommended for use with this feature) is
        // quite bright, so raising the exposure compensation helps
        // bring the scene to a nicer brightness range.
        Exposure { ev100: 13.0 },
        // Tonemapper chosen just because it looked good with the scene, any
        // tonemapper would be fine :)
        Tonemapping::TonyMcMapface,
        // Bloom gives the sun a much more natural look.
        Bloom::NATURAL,
        // Enables the atmosphere to drive reflections and ambient lighting (IBL) for this view
        AtmosphereEnvironmentMapLight::default(),
        VolumetricFog {
            ambient_intensity: 0.0,
            ..default()
        },
        Msaa::Off,
        TemporalAntiAliasing::default(),
        ScreenSpaceReflections::default(),
    ));

    commands.spawn((
        DirectionalLight {
            shadows_enabled: true,
            // lux::RAW_SUNLIGHT is recommended for use with this feature, since
            // other values approximate sunlight *post-scattering* in various
            // conditions. RAW_SUNLIGHT in comparison is the illuminance of the
            // sun unfiltered by the atmosphere, so it is the proper input for
            // sunlight to be filtered by the atmosphere.
            illuminance: light_consts::lux::RAW_SUNLIGHT,
            ..default()
        },
        Transform::from_xyz(1.0, 2.0, 0.0).looking_at(Vec3::ZERO, Vec3::Y),
        VolumetricLight,
        // CascadeShadowConfigBuilder {
        //     first_cascade_far_bound: 0.3,
        //     maximum_distance: 15.0,
        //     ..default()
        // }
        // .build(),
    ));

    commands.spawn((
        FogVolume::default(),
        Transform::from_scale(Vec3::new(10.0, 1.0, 10.0)).with_translation(Vec3::Y * 0.5),
    ));
}

#[allow(clippy::collapsible_else_if)]
fn setup_city(mut commands: Commands, asset_server: Res<AssetServer>) {
    let crossroad: Handle<Scene> = asset_server
        .load(GltfAssetLabel::Scene(0).from_asset("kenney_roads/road-crossroad-path.glb"));
    let straight: Handle<Scene> =
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("kenney_roads/road-straight.glb"));

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

    let half_size = 20;
    let road_len = 3;

    for x in -half_size..half_size {
        for z in -half_size..half_size {
            if x % road_len == 0 && z % road_len == 0 {
                commands.spawn((
                    SceneRoot(crossroad.clone()),
                    Transform::from_xyz(x as f32, 0.0, z as f32),
                ));
            } else if x % road_len == 0 || z % road_len == 0 {
                if x % road_len == 0 {
                    commands.spawn((
                        SceneRoot(straight.clone()),
                        Transform::from_xyz(x as f32, 0.0, z as f32).with_rotation(
                            Quat::from_axis_angle(Vec3::Y, std::f32::consts::FRAC_PI_2),
                        ),
                    ));
                } else {
                    commands.spawn((
                        SceneRoot(straight.clone()),
                        Transform::from_xyz(x as f32, 0.0, z as f32),
                    ));
                }
            } else {
                let rng = rand::random_range(0..buildings.len());
                commands.spawn((
                    SceneRoot(buildings[rng].clone()),
                    Transform::from_xyz(x as f32, 0.0, z as f32),
                ));
            }
        }
    }
}
