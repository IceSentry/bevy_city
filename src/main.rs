use core::f64;

use bevy::camera::Exposure;
use bevy::camera_controller::free_camera::{FreeCamera, FreeCameraPlugin};
use bevy::diagnostic::FrameCount;
use bevy::light::{AtmosphereEnvironmentMapLight, VolumetricFog, VolumetricLight};
use bevy::pbr::{Atmosphere, AtmosphereSettings, ScatteringMedium, ScreenSpaceReflections};
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use noise::{NoiseFn, OpenSimplex};
use rand::rngs::SmallRng;
use rand::{RngExt, SeedableRng};

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "bevy_city".into(),
                    resolution: (1920, 1080).into(),
                    visible: false,
                    ..default()
                }),
                ..default()
            }),
            FreeCameraPlugin,
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(GlobalAmbientLight::NONE)
        .add_systems(Startup, (setup_camera, setup_city))
        .add_systems(Update, make_visible)
        .run();
}

fn make_visible(mut window: Single<&mut Window>, frames: Res<FrameCount>) {
    // The delay may be different for your app or system.
    if frames.0 == 3 {
        // At this point the gpu is ready to show the app so we can make the window visible.
        // Alternatively, you could toggle the visibility in Startup.
        // It will work, but it will have one white frame before it starts rendering
        window.visible = true;
    }
}

fn setup_camera(mut commands: Commands, mut scattering_mediums: ResMut<Assets<ScatteringMedium>>) {
    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 15.0, 50.0).looking_at(Vec3::ZERO, Vec3::Y),
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

    let tree_small: Handle<Scene> = asset_server
        .load(GltfAssetLabel::Scene(0).from_asset("kenney_city_suburban/tree-small.glb"));
    let tree_large: Handle<Scene> = asset_server
        .load(GltfAssetLabel::Scene(0).from_asset("kenney_city_suburban/tree-large.glb"));
    let _planter: Handle<Scene> =
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("kenney_city_suburban/planter.glb"));

    let trees = [tree_small.clone(), tree_large.clone()];

    let path_stones_long: Handle<Scene> = asset_server
        .load(GltfAssetLabel::Scene(0).from_asset("kenney_city_suburban/path-stones-long.glb"));
    let _path_stones_short: Handle<Scene> = asset_server
        .load(GltfAssetLabel::Scene(0).from_asset("kenney_city_suburban/path-stones-short.glb"));

    let low_density_buildings: Vec<Handle<Scene>> = vec![
        asset_server
            .load(GltfAssetLabel::Scene(0).from_asset("kenney_city_suburban/building-type-b.glb")),
        asset_server
            .load(GltfAssetLabel::Scene(0).from_asset("kenney_city_suburban/building-type-f.glb")),
        asset_server
            .load(GltfAssetLabel::Scene(0).from_asset("kenney_city_suburban/building-type-i.glb")),
        asset_server
            .load(GltfAssetLabel::Scene(0).from_asset("kenney_city_suburban/building-type-o.glb")),
        asset_server
            .load(GltfAssetLabel::Scene(0).from_asset("kenney_city_suburban/building-type-u.glb")),
    ];
    let fence: Handle<Scene> =
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("kenney_city_suburban/fence.glb"));

    let mut rng = SmallRng::seed_from_u64(42);
    // TODO better noise
    let noise = OpenSimplex::new(rng.random());

    let mut spawn_city_block = |offset: Vec3| {
        commands.spawn((
            SceneRoot(crossroad.clone()),
            Transform::from_translation(offset),
        ));

        // X roads
        commands.spawn((
            SceneRoot(straight.clone()),
            Transform::from_translation(Vec3::new(2.75, 0.0, 0.0) + offset)
                .with_scale(Vec3::new(4.5, 1.0, 1.0)),
        ));

        // Z roads
        commands.spawn((
            SceneRoot(straight.clone()),
            Transform::from_translation(Vec3::new(0.0, 0.0, 2.0) + offset)
                .with_scale(Vec3::new(3.0, 1.0, 1.0))
                .with_rotation(Quat::from_axis_angle(Vec3::Y, std::f32::consts::FRAC_PI_2)),
        ));

        // TODO if city block is mid or low density use green tile
        let tile_scale = Vec3::new(4.5, 1.0, 3.0);
        commands.spawn((
            SceneRoot(tile.clone()),
            Transform::from_translation(Vec3::new(0.5, -0.5005, 0.5) + tile_scale / 2.0 + offset)
                .with_scale(tile_scale),
        ));

        let scale = 0.025;
        let density =
            noise.get([offset.x as f64 * scale, offset.z as f64 * scale, 0.0]) * 0.5 + 0.5;

        let low_density = 0.6;
        let medium_density = 0.7;

        if density < low_density {
            // low denisty
            for z in 0..=8 {
                commands.spawn((
                    SceneRoot(tree_small.clone()),
                    Transform::from_translation(
                        Vec3::new(0.75, 0.0, 0.75 + z as f32 * 0.3) + offset,
                    ),
                ));
                commands.spawn((
                    SceneRoot(tree_small.clone()),
                    Transform::from_translation(
                        Vec3::new(4.75, 0.0, 0.75 + z as f32 * 0.3) + offset,
                    ),
                ));
            }
            for i in 0..=6 {
                commands.spawn((
                    SceneRoot(fence.clone()),
                    Transform::from_translation(
                        Vec3::new(2.75, 0.0, 0.75 + i as f32 * 0.4) + offset,
                    )
                    .with_rotation(Quat::from_axis_angle(Vec3::Y, std::f32::consts::FRAC_PI_2)),
                ));
            }
            for x in 1..=2 {
                let x_factor = 1.8;
                commands.spawn((
                    SceneRoot(
                        low_density_buildings[rng.random_range(0..low_density_buildings.len())]
                            .clone(),
                    ),
                    Transform::from_translation(Vec3::new(x as f32 * x_factor, 0.0, 1.25) + offset),
                ));
                commands.spawn((
                    SceneRoot(
                        low_density_buildings[rng.random_range(0..low_density_buildings.len())]
                            .clone(),
                    ),
                    Transform::from_translation(Vec3::new(x as f32 * x_factor, 0.0, 2.75) + offset)
                        .with_rotation(Quat::from_axis_angle(Vec3::Y, std::f32::consts::PI)),
                ));
            }
        } else if density < medium_density {
            // medium dcnsity
            // TODO randomize what is spawned in the alley

            // Use the same tree for the entire alley
            let tree = trees[rng.random_range(0..2)].clone();

            // Squish things a little so we can spawn more buildings
            let x_factor = 0.9;
            for x in 1..=5 {
                commands.spawn((
                    SceneRoot(buildings[rng.random_range(0..buildings.len())].clone()),
                    Transform::from_translation(Vec3::new(x as f32 * x_factor, 0.0, 1.0) + offset),
                ));
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
                    commands.spawn((
                        SceneRoot(tree.clone()),
                        Transform::from_translation(
                            Vec3::new(tree_x + x as f32 * x_factor, 0.0, 2.25) + offset,
                        ),
                    ));
                }
                commands.spawn((
                    SceneRoot(buildings[rng.random_range(0..buildings.len())].clone()),
                    Transform::from_translation(Vec3::new(x as f32 * x_factor, 0.0, 3.0) + offset)
                        .with_rotation(Quat::from_axis_angle(Vec3::Y, std::f32::consts::PI)),
                ));
            }

            // path
            for x in 0..=10 {
                commands.spawn((
                    SceneRoot(path_stones_long.clone()),
                    Transform::from_translation(
                        Vec3::new(
                            0.75 + (x as f32 * 0.4),
                            0.02, /*+ 0.02 * x as f32*/
                            2.0,
                        ) + offset,
                    )
                    .with_scale(Vec3::new(1.0, 2.0, 1.0))
                    .with_rotation(Quat::from_axis_angle(Vec3::Y, std::f32::consts::FRAC_PI_2)),
                ));
            }
        } else {
            // high density

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
        }
    };

    let size = 10;
    for x in -size..=size {
        for z in -size..=size {
            spawn_city_block(Vec3::new(x as f32 * 5.5, 0.0, z as f32 * 4.0));
        }
    }

    // {
    //     let mut density_min = f64::MAX;
    //     let mut density_max = f64::MIN;
    //
    //     use std::fmt::Write;
    //     let size = 512;
    //     let mut image = String::new();
    //     let _ = writeln!(image, "P3");
    //     let _ = writeln!(image, "{} {}", size, size);
    //     let _ = writeln!(image, "255");
    //     let scale = 0.005;
    //     for y in 0..size {
    //         for x in 0..size {
    //             let density = perlin.get([x as f64 * scale, y as f64 * scale, 0.0]) * 0.5 + 0.5;
    //             density_min = density_min.min(density);
    //             density_max = density_max.max(density);
    //             let _ = writeln!(image, "{d} {d} {d}", d = (density * 255.99) as u8);
    //         }
    //         // let _ = writeln!(image);
    //     }
    //     let _ = std::fs::write("./density.ppm", image);
    //     println!("density range: {density_min}..{density_max}");
    // }
}
