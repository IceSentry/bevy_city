use core::f64;

use argh::FromArgs;
use bevy::{
    anti_alias::taa::TemporalAntiAliasing,
    camera::{Exposure, Hdr},
    camera_controller::free_camera::{FreeCamera, FreeCameraPlugin},
    color::palettes::css::WHITE,
    dev_tools::fps_overlay::{FpsOverlayConfig, FpsOverlayPlugin, FrameTimeGraphConfig},
    diagnostic::FrameCount,
    feathers::{FeathersPlugins, dark_theme::create_dark_theme, theme::UiTheme},
    light::{
        Atmosphere, AtmosphereEnvironmentMapLight, VolumetricFog, VolumetricLight,
        atmosphere::ScatteringMedium,
    },
    pbr::{
        AtmosphereSettings, ContactShadows,
        wireframe::{WireframeConfig, WireframePlugin},
    },
    post_process::bloom::Bloom,
    prelude::*,
    render::{
        RenderPlugin,
        settings::{WgpuFeatures, WgpuSettings},
    },
    window::{PresentMode, WindowResolution},
    winit::WinitSettings,
};

use noise::{NoiseFn, OpenSimplex};
use rand::{RngExt, SeedableRng, rngs::SmallRng};

use crate::{
    assets::{CityAssets, load_assets},
    generate_city::spawn_city,
    settings::{Settings, setup_settings_ui},
};

mod assets;
mod generate_city;
mod settings;

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

#[derive(FromArgs, Resource, Clone)]
/// Config
pub struct Args {
    /// seed
    #[argh(option, default = "42")]
    seed: u64,

    /// size
    #[argh(option, default = "30")]
    size: u32,
}

fn main() {
    let args: Args = argh::from_env();

    App::new()
        .add_plugins((
            DefaultPlugins.set(WindowPlugin {
                primary_window: Some(Window {
                    title: "bevy_city".into(),
                    resolution: WindowResolution::new(1920, 1080).with_scale_factor_override(1.0),
                    present_mode: PresentMode::AutoNoVsync,
                    visible: false,
                    ..default()
                }),
                ..default()
            }),
            // .set(RenderPlugin {
            //     render_creation: WgpuSettings {
            //         features: WgpuFeatures::POLYGON_MODE_LINE,
            //         ..default()
            //     }
            //     .into(),
            //     ..default()
            // }),
            FreeCameraPlugin,
            FeathersPlugins,
            WireframePlugin::default(),
            // FpsOverlayPlugin {
            //     config: FpsOverlayConfig {
            //         text_config: TextFont {
            //             font_size: FontSize::Px(32.0),
            //             ..default()
            //         },
            //         // We can also change color of the overlay
            //         text_color: WHITE.into(),
            //         refresh_interval: core::time::Duration::from_millis(100),
            //         enabled: true,
            //         frame_time_graph_config: FrameTimeGraphConfig {
            //             enabled: true,
            //             // The minimum acceptable fps
            //             min_fps: 30.0,
            //             // The target fps
            //             target_fps: 144.0,
            //         },
            //     },
            // },
        ))
        .insert_resource(args.clone())
        .init_resource::<Settings>()
        .insert_resource(UiTheme(create_dark_theme()))
        .insert_resource(WinitSettings::continuous())
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
                setup,
                setup_settings_ui,
                load_assets,
                setup_city.after(load_assets),
            ),
        )
        .add_systems(Startup, (spawn_stats_ui))
        .add_systems(Update, (make_visible, simulate_cars, update_stats_ui))
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

fn setup(mut commands: Commands, mut scattering_mediums: ResMut<Assets<ScatteringMedium>>) {
    commands.spawn((
        Camera3d::default(),
        Hdr,
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
        Msaa::Off,
        TemporalAntiAliasing::default(),
        ContactShadows::default(),
    ));

    commands.spawn((
        DirectionalLight {
            shadow_maps_enabled: Settings::default().shadow_maps_enabled,
            contact_shadows_enabled: Settings::default().contact_shadows_enabled,
            illuminance: light_consts::lux::RAW_SUNLIGHT,
            ..default()
        },
        Transform::from_xyz(1.0, 0.15, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));
}

fn spawn_stats_ui(mut commands: Commands) {
    commands
        .spawn((
            Node {
                position_type: PositionType::Absolute,
                bottom: Val::Px(10.0),
                left: Val::Px(10.0),
                padding: UiRect::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(Color::srgba(0.0, 0.0, 0.0, 0.75)),
        ))
        .with_children(|parent| {
            parent.spawn((
                Text::new(""),
                TextFont {
                    font_size: FontSize::Px(20.0),
                    ..default()
                },
                TextColor(Color::WHITE),
                StatsText,
            ));
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
        Hdr,
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
        Msaa::Off,
        TemporalAntiAliasing::default(),
        // ScreenSpaceReflections::default(),
    ));

    commands.spawn((
        DirectionalLight {
            shadow_maps_enabled: false,
            illuminance: light_consts::lux::RAW_SUNLIGHT,
            ..default()
        },
        Transform::from_xyz(1.0, 0.15, 1.0).looking_at(Vec3::ZERO, Vec3::Y),
        VolumetricLight,
    ));
}

fn setup_city(mut commands: Commands, assets: Res<CityAssets>, args: Res<Args>) {
    spawn_city(&mut commands, &assets, args.seed, args.size);
}

#[derive(Component)]
struct Road {
    start: Vec3,
    end: Vec3,
}

#[derive(Component)]
struct Car {
    offset: Vec3,
    distance_traveled: f32,
    dir: f32,
}

fn simulate_cars(
    settings: Res<Settings>,
    roads: Query<(&Road, &Transform, &Children), Without<Car>>,
    mut cars: Query<(&mut Car, &mut Transform), Without<Road>>,
    time: Res<Time>,
) {
    if !settings.simulate_cars {
        return;
    }
    let speed = 1.5;

    for (road, _, children) in &roads {
        for child in children {
            let Ok((mut car, mut car_transform)) = cars.get_mut(*child) else {
                continue;
            };

            car.distance_traveled += speed * time.delta_secs();
            let road_len = (road.end - road.start).length();
            if car.distance_traveled > road_len {
                car.distance_traveled = 0.0;
            }
            let direction = (road.end - road.start).normalize() * car.dir;

            let progress = car.distance_traveled / road_len;
            car_transform.translation = (road.start + car.offset) + direction * road_len * progress;
        }
    }
}
