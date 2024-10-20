mod animation;
mod plugin;

use animation::*;
use avian2d::{math::*, prelude::*};
use bevy::{
    prelude::*,
    render::{render_asset::RenderAssetUsages, render_resource::PrimitiveTopology},
    sprite::MaterialMesh2dBundle,
};
use plugin::*;

fn main() {
    App::new()
        .add_plugins((
            DefaultPlugins.set(ImagePlugin::default_nearest()), // prevents blurry sprites according to example
            // Add physics plugins and specify a units-per-meter scaling factor, 1 meter = 20 pixels.
            // The unit allows the engine to tune its parameters for the scale of the world, improving stability.
            PhysicsPlugins::default().with_length_unit(20.0),
            CharacterControllerPlugin,
        ))
        .insert_resource(ClearColor(Color::srgb(0.05, 0.05, 0.1)))
        .insert_resource(Gravity(Vector::NEG_Y * 800.0))
        .add_systems(Startup, setup)
        .add_systems(Update, animate_sprite)
        .run();
}

fn setup(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
    asset_server: Res<AssetServer>,
    mut texture_atlas_layouts: ResMut<Assets<TextureAtlasLayout>>,
) {
    // Player
    let texture = asset_server.load("textures/gabe-idle-run.png");
    let layout = TextureAtlasLayout::from_grid(UVec2::splat(24), 7, 1, None, None);
    let texture_atlas_layout = texture_atlas_layouts.add(layout);
    // Use only the subset of sprites in the sheet that make up the run animation
    let animation_indices = AnimationIndices { first: 1, last: 6 };

    commands.spawn((
        SpriteBundle {
            transform: Transform::from_xyz(0.0, -100.0, 0.0),
            texture,
            ..default()
        },
        TextureAtlas {
            layout: texture_atlas_layout,
            index: animation_indices.first,
        },
        animation_indices,
        AnimationTimer(Timer::from_seconds(0.1, TimerMode::Repeating)),
        // MaterialMesh2dBundle {
        //     mesh: meshes.add(Capsule2d::new(12.5, 20.0)).into(),
        //     material: materials.add(Color::srgb(0.2, 0.7, 0.9)),
        //     transform: Transform::from_xyz(0.0, -100.0, 0.0),
        //     ..default()
        // },
        CharacterControllerBundle::new(Collider::capsule(12.5, 20.0)).with_movement(
            1250.0,
            0.92,
            400.0,
            (30.0 as Scalar).to_radians(),
        ),
        Friction::ZERO.with_combine_rule(CoefficientCombine::Min),
        Restitution::ZERO.with_combine_rule(CoefficientCombine::Min),
        ColliderDensity(2.0),
        GravityScale(1.5),
    ));

    // A cube to move around
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.0, 0.4, 0.7),
                custom_size: Some(Vec2::new(30.0, 30.0)),
                ..default()
            },
            transform: Transform::from_xyz(50.0, -100.0, 0.0),
            ..default()
        },
        RigidBody::Dynamic,
        Collider::rectangle(30.0, 30.0),
    ));

    // Platforms
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.7, 0.7, 0.8),
                custom_size: Some(Vec2::new(1100.0, 50.0)),
                ..default()
            },
            transform: Transform::from_xyz(0.0, -175.0, 0.0),
            ..default()
        },
        RigidBody::Static,
        Collider::rectangle(1100.0, 50.0),
    ));
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.7, 0.7, 0.8),
                custom_size: Some(Vec2::new(300.0, 25.0)),
                ..default()
            },
            transform: Transform::from_xyz(175.0, -35.0, 0.0),
            ..default()
        },
        RigidBody::Static,
        Collider::rectangle(300.0, 25.0),
    ));
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.7, 0.7, 0.8),
                custom_size: Some(Vec2::new(300.0, 25.0)),
                ..default()
            },
            transform: Transform::from_xyz(-175.0, 0.0, 0.0),
            ..default()
        },
        RigidBody::Static,
        Collider::rectangle(300.0, 25.0),
    ));
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.7, 0.7, 0.8),
                custom_size: Some(Vec2::new(150.0, 80.0)),
                ..default()
            },
            transform: Transform::from_xyz(475.0, -110.0, 0.0),
            ..default()
        },
        RigidBody::Static,
        Collider::rectangle(150.0, 80.0),
    ));
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.7, 0.7, 0.8),
                custom_size: Some(Vec2::new(150.0, 80.0)),
                ..default()
            },
            transform: Transform::from_xyz(-475.0, -110.0, 0.0),
            ..default()
        },
        RigidBody::Static,
        Collider::rectangle(150.0, 80.0),
    ));

    // Ramps

    let mut ramp_mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );

    ramp_mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![[-125.0, 80.0, 0.0], [-125.0, 0.0, 0.0], [125.0, 0.0, 0.0]],
    );

    let ramp_collider = Collider::triangle(
        Vector::new(-125.0, 80.0),
        Vector::NEG_X * 125.0,
        Vector::X * 125.0,
    );

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(ramp_mesh).into(),
            material: materials.add(Color::srgb(0.4, 0.4, 0.5)),
            transform: Transform::from_xyz(-275.0, -150.0, 0.0),
            ..default()
        },
        RigidBody::Static,
        ramp_collider,
    ));

    let mut ramp_mesh = Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    );

    ramp_mesh.insert_attribute(
        Mesh::ATTRIBUTE_POSITION,
        vec![[20.0, -40.0, 0.0], [20.0, 40.0, 0.0], [-20.0, -40.0, 0.0]],
    );

    let ramp_collider = Collider::triangle(
        Vector::new(20.0, -40.0),
        Vector::new(20.0, 40.0),
        Vector::new(-20.0, -40.0),
    );

    commands.spawn((
        MaterialMesh2dBundle {
            mesh: meshes.add(ramp_mesh).into(),
            material: materials.add(Color::srgb(0.4, 0.4, 0.5)),
            transform: Transform::from_xyz(380.0, -110.0, 0.0),
            ..default()
        },
        RigidBody::Static,
        ramp_collider,
    ));

    // Camera
    commands.spawn(Camera2dBundle::default());

    // Add two walls to jump between
    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.7, 0.7, 0.8),
                custom_size: Some(Vec2::new(50.0, 300.0)),
                ..default()
            },
            transform: Transform::from_xyz(-50.0, 0.0, 0.0),
            ..default()
        },
        RigidBody::Static,
        Collider::rectangle(50.0, 300.0),
    ));

    commands.spawn((
        SpriteBundle {
            sprite: Sprite {
                color: Color::srgb(0.7, 0.7, 0.8),
                custom_size: Some(Vec2::new(50.0, 300.0)),
                ..default()
            },
            transform: Transform::from_xyz(50.0, 0.0, 0.0),
            ..default()
        },
        RigidBody::Static,
        Collider::rectangle(50.0, 300.0),
    ));
}
