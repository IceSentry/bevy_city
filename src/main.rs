use core::f64;

use bevy::camera::Exposure;
use bevy::camera_controller::free_camera::{FreeCamera, FreeCameraPlugin};
use bevy::color::palettes::css::WHITE;
use bevy::dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin, FrameTimeGraphConfig};
use bevy::diagnostic::FrameCount;
use bevy::light::{AtmosphereEnvironmentMapLight, VolumetricFog, VolumetricLight};
use bevy::pbr::wireframe::{WireframeConfig, WireframePlugin};
use bevy::pbr::{Atmosphere, AtmosphereSettings, ScatteringMedium, ScreenSpaceReflections};
use bevy::post_process::bloom::Bloom;
use bevy::prelude::*;
use bevy::render::settings::{WgpuFeatures, WgpuSettings};
use bevy::render::RenderPlugin;
use noise::{NoiseFn, OpenSimplex};
use rand::rngs::SmallRng;
use rand::{RngExt, SeedableRng};

use crate::low_density::{load_low_density_buildings, spawn_low_density, LowDensityBuildings};
use crate::medium_density::{
    load_medium_density_buildings, spawn_medium_density, MediumDensityBuildings,
};
use crate::skyscrapers::{load_skyscrapers, spawn_high_density, SkyscraperBuildings};

mod low_density;
mod medium_density;
mod skyscrapers;

#[derive(Component)]
struct Car {
    road_segment: Entity,
    speed: f32,
    distance_traveled: f32,
}

#[derive(Component, Clone, Copy)]
struct RoadSegment {
    start: Vec3,
    _end: Vec3,
    direction: Vec3,
    length: f32,
}

impl RoadSegment {
    fn new(start: Vec3, end: Vec3) -> Self {
        let direction = (end - start).normalize();
        let length = (end - start).length();
        RoadSegment {
            start,
            _end: end,
            direction,
            length,
        }
    }
}

#[derive(Resource, Default)]
struct SceneStats {
    cars_spawned: u32,
    low_density_buildings: u32,
    medium_density_buildings: u32,
    skyscrapers: u32,
    road_segments: u32,
    trees: u32,
}

#[derive(Component)]
struct StatsText;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "bevy_city".into(),
                        resolution: (1920, 1080).into(),
                        visible: false,
                        ..default()
                    }),
                    ..default()
                })
                .set(RenderPlugin {
                    render_creation: WgpuSettings {
                        features: WgpuFeatures::POLYGON_MODE_LINE,
                        ..default()
                    }
                    .into(),
                    ..default()
                }),
            FreeCameraPlugin,
            WireframePlugin::default(),
            FpsOverlayPlugin {
                config: FpsOverlayConfig {
                    text_config: TextFont {
                        font_size: 32.0,
                        ..default()
                    },
                    // We can also change color of the overlay
                    text_color: WHITE.into(),
                    refresh_interval: core::time::Duration::from_millis(100),
                    enabled: true,
                    frame_time_graph_config: FrameTimeGraphConfig {
                        enabled: true,
                        // The minimum acceptable fps
                        min_fps: 30.0,
                        // The target fps
                        target_fps: 144.0,
                    },
                },
            },
        ))
        .insert_resource(ClearColor(Color::BLACK))
        .insert_resource(GlobalAmbientLight::NONE)
        .insert_resource(WireframeConfig {
            global: false,
            default_color: WHITE.into(),
        })
        .init_resource::<SceneStats>()
        .add_systems(
            Startup,
            (
                (
                    load_low_density_buildings,
                    load_medium_density_buildings,
                    load_skyscrapers,
                    load_ground_tiles,
                    load_cars,
                )
                    .before(setup_city),
                setup_city,
            ),
        )
        .add_systems(Startup, (setup_camera, spawn_stats_ui))
        .add_systems(
            Update,
            (make_visible, toggle_wireframe, move_cars, update_stats_ui),
        )
        // .add_observer(generate_variations)
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

fn toggle_wireframe(
    keyboard_input: Res<ButtonInput<KeyCode>>,
    mut config: ResMut<WireframeConfig>,
) {
    if keyboard_input.just_pressed(KeyCode::KeyZ) {
        config.global = !config.global;
    }
}

fn move_cars(
    mut cars: Query<(&mut Car, &mut Transform)>,
    segments: Query<&RoadSegment>,
    time: Res<Time>,
) {
    for (mut car, mut transform) in cars.iter_mut() {
        if let Ok(segment) = segments.get(car.road_segment) {
            car.distance_traveled += car.speed * time.delta_secs();

            if car.distance_traveled > segment.length {
                car.distance_traveled = 0.0;
            }

            let progress = car.distance_traveled / segment.length;
            let new_pos = segment.start + segment.direction * segment.length * progress;
            transform.translation = new_pos;
        }
    }
}

fn spawn_stats_ui(mut commands: Commands) {
    commands
        .spawn(Node {
            width: Val::Percent(100.0),
            height: Val::Percent(100.0),
            ..default()
        })
        .with_children(|parent| {
            parent
                .spawn((
                    Node {
                        position_type: PositionType::Absolute,
                        bottom: Val::Px(10.0),
                        left: Val::Px(10.0),
                        // margin: UiRect::all(Val::Px(25.0)),
                        padding: UiRect::all(Val::Px(10.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.75)),
                ))
                .with_children(|parent| {
                    parent.spawn((
                        Text::new(""),
                        TextFont {
                            font_size: 20.0,
                            ..default()
                        },
                        TextColor(Color::WHITE),
                        StatsText,
                    ));
                });
        });
}

fn format_large_number(value: u32) -> String {
    let mut s = String::new();
    for (i, char) in value.to_string().chars().rev().enumerate() {
        if i % 3 == 0 && i != 0 {
            s.insert(0, ',');
        }
        s.insert(0, char);
    }
    s
}

fn update_stats_ui(
    mut stats_text: Single<&mut Text, With<StatsText>>,
    stats: Res<SceneStats>,
    entities: Query<Entity>,
) {
    let total_entities = entities.iter().count();
    stats_text.0 = format!(
        "Cars: {}\nLow Density: {}\nMedium Density: {}\nSkyscrapers: {}\nRoad Segments: {}\nTrees: {}\nTotal spawned mesh: {}\nTotal Entities: {}",
        format_large_number(stats.cars_spawned),
        format_large_number(stats.low_density_buildings),
        format_large_number(stats.medium_density_buildings),
        format_large_number(stats.skyscrapers),
        format_large_number(stats.road_segments),
        format_large_number(stats.trees),
        format_large_number(
            stats.cars_spawned
                + stats.low_density_buildings
                + stats.medium_density_buildings
                + stats.skyscrapers
                + stats.road_segments
                + stats.trees
        ),
        format_large_number(total_entities as u32)
    );
}

fn setup_camera(mut commands: Commands, mut scattering_mediums: ResMut<Assets<ScatteringMedium>>) {
    // camera
    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(15.0, 10.0, 20.0).looking_at(Vec3::ZERO, Vec3::Y),
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

#[derive(Resource)]
struct CarAssets {
    cars: Vec<Handle<Scene>>,
}

impl CarAssets {
    fn random_car<R: RngExt>(&self, rng: &mut R) -> Handle<Scene> {
        self.cars[rng.random_range(0..self.cars.len())].clone()
    }
}

fn load_cars(mut commands: Commands, asset_server: Res<AssetServer>) {
    let cars = [
        "hatchback-sports",
        "suv",
        "suv-luxury",
        "sedan",
        "sedan-sports",
        "truck",
        "truck-flat",
        "van",
    ]
    .iter()
    .map(|t| asset_server.load(GltfAssetLabel::Scene(0).from_asset(format!("kenney_cars/{t}.glb"))))
    .collect::<Vec<Handle<Scene>>>();

    commands.insert_resource(CarAssets { cars });
}

#[derive(Resource)]
struct GroundTiles {
    mesh: Handle<Mesh>,
    default_material: Handle<StandardMaterial>,
    grass_material: Handle<StandardMaterial>,
}

fn load_ground_tiles(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let mesh = asset_server.load(
        GltfAssetLabel::Primitive {
            mesh: 0,
            primitive: 0,
        }
        .from_asset("kenney_roads/tile-low.glb"),
    );
    let default_material = asset_server.load(
        GltfAssetLabel::Material {
            index: 0,
            is_scale_inverted: false,
        }
        .from_asset("kenney_roads/tile-low.glb"),
    );
    let grass_material = materials.add(StandardMaterial::from_color(Color::srgb_u8(97, 203, 139)));
    commands.insert_resource(GroundTiles {
        mesh,
        default_material,
        grass_material,
    });
}

#[allow(clippy::too_many_arguments)]
fn setup_city(
    mut commands: Commands,
    asset_server: Res<AssetServer>,
    car_assets: Res<CarAssets>,
    low_density_buildings: Res<LowDensityBuildings>,
    medium_density_buildings: Res<MediumDensityBuildings>,
    skyscrapers: Res<SkyscraperBuildings>,
    ground_tile: Res<GroundTiles>,
    mut stats: ResMut<SceneStats>,
) {
    let crossroad: Handle<Scene> = asset_server
        .load(GltfAssetLabel::Scene(0).from_asset("kenney_roads/road-crossroad-path.glb"));
    let straight: Handle<Scene> =
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("kenney_roads/road-straight.glb"));
    let _straight_half: Handle<Scene> = asset_server
        .load(GltfAssetLabel::Scene(0).from_asset("kenney_roads/road-straight-half.glb"));

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

    let fence: Handle<Scene> =
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("kenney_city_suburban/fence.glb"));

    let mut rng = SmallRng::seed_from_u64(42);
    // TODO better noise
    let noise = OpenSimplex::new(rng.random());

    let mut spawn_city_block = |offset: Vec3| {
        spawn_roads_and_cars(
            &mut commands,
            &mut stats,
            &mut rng,
            offset,
            &crossroad,
            &straight,
            &car_assets,
        );

        let ground_tile_scale = Vec3::new(4.5, 1.0, 3.0);

        let noise_scale = 0.025;
        let density = noise.get([
            offset.x as f64 * noise_scale,
            offset.z as f64 * noise_scale,
            0.0,
        ]) * 0.5
            + 0.5;

        let rural = 0.45;
        let low_density = 0.6;
        let medium_density = 0.7;

        if density < rural {
            commands.spawn((
                Mesh3d(ground_tile.mesh.clone()),
                MeshMaterial3d(ground_tile.grass_material.clone()),
                Transform::from_translation(
                    Vec3::new(0.5, -0.5005, 0.5) + ground_tile_scale / 2.0 + offset,
                )
                .with_scale(ground_tile_scale),
            ));
        } else if density < low_density {
            commands.spawn((
                Mesh3d(ground_tile.mesh.clone()),
                MeshMaterial3d(ground_tile.grass_material.clone()),
                Transform::from_translation(
                    Vec3::new(0.5, -0.5005, 0.5) + ground_tile_scale / 2.0 + offset,
                )
                .with_scale(ground_tile_scale),
            ));

            spawn_low_density(
                &mut commands,
                &mut stats,
                &mut rng,
                offset,
                &low_density_buildings,
                tree_small.clone(),
                fence.clone(),
            );
        } else if density < medium_density {
            commands.spawn((
                Mesh3d(ground_tile.mesh.clone()),
                MeshMaterial3d(ground_tile.default_material.clone()),
                Transform::from_translation(
                    Vec3::new(0.5, -0.5005, 0.5) + ground_tile_scale / 2.0 + offset,
                )
                .with_scale(ground_tile_scale),
            ));

            spawn_medium_density(
                &mut commands,
                &mut stats,
                &mut rng,
                offset,
                &medium_density_buildings,
                &trees,
                path_stones_long.clone(),
            );
        } else {
            commands.spawn((
                Mesh3d(ground_tile.mesh.clone()),
                MeshMaterial3d(ground_tile.default_material.clone()),
                Transform::from_translation(
                    Vec3::new(0.5, -0.5005, 0.5) + ground_tile_scale / 2.0 + offset,
                )
                .with_scale(ground_tile_scale),
            ));

            spawn_high_density(&mut commands, &mut stats, &skyscrapers, &mut rng, offset);
        }
    };

    let size = 20;
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

fn spawn_roads_and_cars<R: RngExt>(
    commands: &mut Commands,
    stats: &mut SceneStats,
    mut rng: &mut R,
    offset: Vec3,
    crossroad: &Handle<Scene>,
    straight: &Handle<Scene>,
    cars: &CarAssets,
) {
    commands.spawn((
        SceneRoot(crossroad.clone()),
        Transform::from_translation(offset),
    ));
    stats.road_segments += 1;

    // X roads
    commands.spawn((
        SceneRoot(straight.clone()),
        Transform::from_translation(Vec3::new(2.75, 0.0, 0.0) + offset)
            .with_scale(Vec3::new(4.5, 1.0, 1.0)),
    ));
    stats.road_segments += 1;

    let x_segment_entity = commands
        .spawn(RoadSegment::new(
            Vec3::new(0.3, 0.0, 0.15) + offset,
            Vec3::new(5.2, 0.0, 0.15) + offset,
        ))
        .id();

    let x_segment_reverse_entity = commands
        .spawn(RoadSegment::new(
            Vec3::new(5.2, 0.0, -0.15) + offset,
            Vec3::new(0.3, 0.0, -0.15) + offset,
        ))
        .id();

    // Z roads
    commands.spawn((
        SceneRoot(straight.clone()),
        Transform::from_translation(Vec3::new(0.0, 0.0, 2.0) + offset)
            .with_scale(Vec3::new(3.0, 1.0, 1.0))
            .with_rotation(Quat::from_axis_angle(Vec3::Y, std::f32::consts::FRAC_PI_2)),
    ));
    stats.road_segments += 1;

    let z_segment_entity = commands
        .spawn(RoadSegment::new(
            Vec3::new(-0.15, 0.0, 0.75) + offset,
            Vec3::new(-0.15, 0.0, 3.25) + offset,
        ))
        .id();

    let z_segment_reverse_entity = commands
        .spawn(RoadSegment::new(
            Vec3::new(0.15, 0.0, 3.25) + offset,
            Vec3::new(0.15, 0.0, 0.75) + offset,
        ))
        .id();

    let car_density = 0.75;
    // X cars (positive direction: 0.3 to 5.2)
    for i in 0..9 {
        if rng.random::<f32>() > car_density {
            commands.spawn((
                SceneRoot(cars.random_car(&mut rng)),
                Transform::from_translation(Vec3::new(0.75 + i as f32 * 0.5, 0.0, 0.15) + offset)
                    .with_scale(Vec3::splat(0.15))
                    .with_rotation(Quat::from_axis_angle(
                        Vec3::Y,
                        3.0 * -std::f32::consts::FRAC_PI_2,
                    )),
                Car {
                    road_segment: x_segment_entity,
                    speed: 2.0,
                    distance_traveled: i as f32 * 0.55,
                },
            ));
            stats.cars_spawned += 1;
        }
        // X cars (negative direction: 5.2 to 0.3)
        if rng.random::<f32>() > car_density {
            commands.spawn((
                SceneRoot(cars.random_car(&mut rng)),
                Transform::from_translation(Vec3::new(0.75 + i as f32 * 0.5, 0.0, -0.15) + offset)
                    .with_scale(Vec3::splat(0.15))
                    .with_rotation(Quat::from_axis_angle(Vec3::Y, -std::f32::consts::FRAC_PI_2)),
                Car {
                    road_segment: x_segment_reverse_entity,
                    speed: 2.0,
                    distance_traveled: i as f32 * 0.55,
                },
            ));
            stats.cars_spawned += 1;
        }
    }

    // Z cars (positive direction: 0.75 to 3.25)
    for i in 0..6 {
        if rng.random::<f32>() > car_density {
            commands.spawn((
                SceneRoot(cars.random_car(&mut rng)),
                Transform::from_translation(Vec3::new(-0.15, 0.0, 0.75 + i as f32 * 0.5) + offset)
                    .with_scale(Vec3::splat(0.15)),
                Car {
                    road_segment: z_segment_entity,
                    speed: 2.0,
                    distance_traveled: i as f32 * 0.5,
                },
            ));
            stats.cars_spawned += 1;
        }
        // Z cars (negative direction: 3.25 to 0.75)
        if rng.random::<f32>() > car_density {
            commands.spawn((
                SceneRoot(cars.random_car(&mut rng)),
                Transform::from_translation(Vec3::new(0.15, 0.0, 0.75 + i as f32 * 0.5) + offset)
                    .with_scale(Vec3::splat(0.15))
                    .with_rotation(Quat::from_axis_angle(Vec3::Y, std::f32::consts::PI)),
                Car {
                    road_segment: z_segment_reverse_entity,
                    speed: 2.0,
                    distance_traveled: i as f32 * 0.5,
                },
            ));
            stats.cars_spawned += 1;
        }
    }
}
