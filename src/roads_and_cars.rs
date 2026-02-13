use bevy::prelude::*;
use rand::RngExt;

use crate::{SceneStats, Settings};

#[derive(Component)]
pub struct Car {
    road_segment: Entity,
    speed: f32,
    distance_traveled: f32,
}

#[derive(Component, Clone, Copy)]
pub struct RoadSegment {
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

pub fn move_cars(
    mut cars: Query<(&mut Car, &mut Transform)>,
    segments: Query<&RoadSegment>,
    time: Res<Time>,
    settings: Res<Settings>,
) {
    if !settings.move_cars {
        return;
    }

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

#[derive(Resource)]
pub struct RoadsAndCarsAssets {
    cars: Vec<Handle<Scene>>,
    pub crossroad: Handle<Scene>,
    pub road_straight: Handle<Scene>,
}

impl RoadsAndCarsAssets {
    fn random_car<R: RngExt>(&self, rng: &mut R) -> Handle<Scene> {
        self.cars[rng.random_range(0..self.cars.len())].clone()
    }
}

pub fn load_cars(mut commands: Commands, asset_server: Res<AssetServer>) {
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
    .map(|t| asset_server.load(GltfAssetLabel::Scene(0).from_asset(format!("cars/{t}.glb"))))
    .collect::<Vec<Handle<Scene>>>();

    let crossroad: Handle<Scene> =
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("roads/road-crossroad-path.glb"));
    let road_straight: Handle<Scene> =
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("roads/road-straight.glb"));
    let _straight_half: Handle<Scene> =
        asset_server.load(GltfAssetLabel::Scene(0).from_asset("roads/road-straight-half.glb"));

    commands.insert_resource(RoadsAndCarsAssets {
        cars,
        crossroad,
        road_straight,
    });
}

pub fn spawn_roads_and_cars<R: RngExt>(
    commands: &mut Commands,
    stats: &mut SceneStats,
    mut rng: &mut R,
    offset: Vec3,
    assets: &RoadsAndCarsAssets,
) {
    commands.spawn((
        SceneRoot(assets.crossroad.clone()),
        Transform::from_translation(offset),
    ));
    stats.road_segments += 1;

    // X roads
    commands.spawn((
        SceneRoot(assets.road_straight.clone()),
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
        SceneRoot(assets.road_straight.clone()),
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
                SceneRoot(assets.random_car(&mut rng)),
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
                SceneRoot(assets.random_car(&mut rng)),
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
                SceneRoot(assets.random_car(&mut rng)),
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
                SceneRoot(assets.random_car(&mut rng)),
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
