use bevy::{core_pipeline::tonemapping::Tonemapping, prelude::*, render::camera::ScalingMode};

use bevy::{
    pbr::{MaterialPipeline, MaterialPipelineKey},
    reflect::TypePath,
    render::{
        mesh::{MeshVertexBufferLayout, PrimitiveTopology},
        render_resource::{
            AsBindGroup, PolygonMode, RenderPipelineDescriptor, ShaderRef,
            SpecializedMeshPipelineError,
        },
    },
};

use bevy::time::common_conditions::on_timer;
// use bevy_inspector_egui::quick::WorldInspectorPlugin;
// use bevy_mod_picking::{DefaultPickingPlugins, PickableBundle};
use bevy_mod_reqwest::*;
use bevy_panorbit_camera::*;
use serde::Deserialize;
use std::time::Duration;

const BLOCK_SPEED: f32 = 0.2;
const TX_SPACING: f32 = 0.05;

#[derive(Deserialize)]
struct TransactionResponse {
    #[serde(rename = "blockNumber")]
    block_number: String,
    #[serde(rename = "transactionIndex")]
    index: String,
    gas: String,
}

#[derive(Deserialize)]
struct BlockResponse {
    #[serde(rename = "gasLimit")]
    gas_limit: String,
    #[serde(rename = "gasUsed")]
    gas_used: String,
    number: String,
    transactions: Vec<TransactionResponse>,
}

#[derive(Deserialize)]
struct Response {
    result: BlockResponse,
}

#[derive(Component, Debug, Reflect)]
#[reflect(Component)]
struct Block {
    number: u64,
    gas_limit: u64,
    gas_used: u64,
}

#[derive(Component, Debug, Reflect, Default)]
#[reflect(Component)]
struct Transaction {
    block_number: u64,
    gas: u64,
    index: u64,
}

impl Default for Block {
    fn default() -> Self {
        Block {
            number: 0,
            gas_limit: 0,
            gas_used: 0,
        }
    }
}

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::rgb_u8(47, 72, 88)))
        .register_type::<Block>()
        .add_systems(Startup, setup)
        .add_plugins(PanOrbitCameraPlugin)
        // .add_plugins(DefaultPickingPlugins)
        .add_plugins((
            DefaultPlugins,
            // WorldInspectorPlugin::default(),
            ReqwestPlugin,
            MaterialPlugin::<LineMaterial>::default(),
        ))
        .insert_resource(AmbientLight {
            color: Color::WHITE,
            brightness: 1.0,
        })
        .add_systems(
            Update,
            (
                send_requests.run_if(on_timer(Duration::from_secs(2))),
                handle_responses.run_if(on_timer(Duration::from_secs(1))),
            ),
        )
        .add_systems(Update, block_movement)
        .run();
}

/// set up a simple 3D scene
fn setup(
    mut commands: Commands,
    // mut meshes: ResMut<Assets<Mesh>>,
    // mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // circular base
    // commands.spawn(PbrBundle {
    //     mesh: meshes.add(shape::Circle::new(400.0).into()),
    //     material: materials.add(Color::GRAY.into()),
    //     transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    //     ..default()
    // });
    // light
    // commands.spawn(PointLightBundle {
    //     point_light: PointLight {
    //         intensity: 200.0,
    //         shadows_enabled: true,
    //         ..default()
    //     },
    //     transform: Transform::from_xyz(4.0, 6.0, 4.0),
    //     ..default()
    // });

    commands.spawn((
        Camera3dBundle {
            projection: OrthographicProjection {
                near: 0.0,
                far: 500.0,
                scale: 12.5,
                scaling_mode: ScalingMode::FixedVertical(0.8),
                ..default()
            }
            .into(),
            camera: Camera { ..default() },
            tonemapping: Tonemapping::TonyMcMapface, // 2. Using a tonemapper that desaturates to white is recommended
            transform: Transform::from_xyz(4.0, 4.0, 6.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        PanOrbitCamera::default(),
    ));
}

fn block_movement(mut enemy_query: Query<&mut Transform, With<Block>>, time: Res<Time>) {
    for mut transform in enemy_query.iter_mut() {
        let direction = Vec3::new(0.0, 0.0, -1.0);
        transform.translation += direction * BLOCK_SPEED * time.delta_seconds();
    }
}

fn send_requests(mut commands: Commands, reqwest: Res<ReqwestClient>) {
    let url = "https://mainnet.infura.io/v3/6fffe7dc6c6c42459d5443592d3c3afc";

    let req = reqwest
        .0
        .post(url)
        .json(&serde_json::json!({
            "jsonrpc": "2.0",
            "method": "eth_getBlockByNumber",
            "params": ["latest", true],
            "id": 1,
        }))
        .build()
        .unwrap();
    let req = ReqwestRequest::new(req);
    commands.spawn(req);
}

fn handle_responses(
    mut commands: Commands,
    results: Query<(Entity, &ReqwestBytesResult)>,
    query: Query<&Block>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut line_materials: ResMut<Assets<LineMaterial>>,
) {
    for (e, res) in results.iter() {
        let a: Response = serde_json::from_slice(res.as_ref().unwrap()).unwrap();
        let mut previous_position = Vec3::ZERO;

        if let Ok(number) = u64::from_str_radix(&a.result.number[2..], 16) {
            if let Ok(gas_limit) = u64::from_str_radix(&a.result.gas_limit[2..], 16) {
                if let Ok(gas_used) = u64::from_str_radix(&a.result.gas_used[2..], 16) {
                    let block_exists = query.iter().any(|block| block.number == number);

                    if block_exists {
                        println!("Already have this block");
                    } else {
                        println!("spawning new block {}", number);
                        let ratio = gas_used as f32 / gas_limit as f32;

                        let new_height = 1.0 * ratio;
                        let center_translation = (-0.5 + new_height / 2.0) - 0.01;

                        println!("ratio {}", ratio);
                        commands
                            .spawn(Block {
                                number,
                                gas_limit,
                                gas_used,
                            })
                            .insert(PbrBundle {
                                mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                                material: materials.add(Color::rgb_u8(51, 102, 153).into()),
                                transform: Transform::from_xyz(0.0, 0.5, 0.0),
                                ..default()
                            })
                            // .insert(PickableBundle::default())
                            .with_children(|parent| {
                                parent.spawn(PbrBundle {
                                    mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                                    material: materials.add(StandardMaterial {
                                        base_color: Color::rgb_u8(134, 187, 216), // 4. Put something bright in a dark environment to see the effect
                                        ..default()
                                    }),
                                    // transform: Transform::from_xyz(0.0, center_translation, 0.0).with_scale(Vec3::new(0.99, ratio, 0.99)),
                                    transform: Transform::from_xyz(0.0, center_translation, 0.0)
                                        .with_scale(Vec3::new(1.01, ratio, 1.01)),
                                    ..default()
                                });

                                let mut offset = 1.0 / 2.0 - TX_SPACING;
                                // spawn cubes for each transaction spaced vertically
                                for (i, t) in a.result.transactions.iter().enumerate() {
                                    let gas = u64::from_str_radix(&t.gas[2..], 16).unwrap();
                                    let index = u64::from_str_radix(&t.index[2..], 16).unwrap();
                                    let tx_ratio = gas as f32 / gas_limit as f32;
                                    let tx_translation = offset - (tx_ratio / 2.0);
                                    let current_position = Vec3::new(0.0, tx_translation, 0.0);
                                    let limited_value = f32::min(f32::max(tx_ratio, 0.05), 1.0);
                                    parent.spawn(PbrBundle {
                                        mesh: meshes.add(Mesh::from(shape::Cube { size: ratio })),
                                        material: materials.add(StandardMaterial {
                                            base_color: Color::rgb_u8(134, 187, 216), // 4. Put something bright in a dark environment to see the effect
                                            ..default()
                                        }),
                                        transform: Transform::from_xyz(0.0, tx_translation, 0.0)
                                            .with_scale(limited_value * Vec3::new(1.0, 1.0, 1.0)),
                                        ..default()
                                    });

                                    parent.spawn(MaterialMeshBundle {
                                        mesh: meshes.add(Mesh::from(LineList {
                                            lines: vec![(previous_position, current_position)],
                                        })),
                                        material: line_materials.add(LineMaterial {
                                            color: Color::rgb_u8(134, 187, 216),
                                        }),
                                        ..default()
                                    });

                                    offset -= tx_ratio / 2.0 - TX_SPACING;
                                    previous_position = current_position;
                                }
                            });
                    }
                }
            }
        }

        // Done with this entity
        commands.entity(e).despawn_recursive();
    }
}

#[derive(Asset, TypePath, Default, AsBindGroup, Debug, Clone)]
struct LineMaterial {
    #[uniform(0)]
    color: Color,
}

impl Material for LineMaterial {
    fn fragment_shader() -> ShaderRef {
        "shaders/line_material.wgsl".into()
    }

    fn specialize(
        _pipeline: &MaterialPipeline<Self>,
        descriptor: &mut RenderPipelineDescriptor,
        _layout: &MeshVertexBufferLayout,
        _key: MaterialPipelineKey<Self>,
    ) -> Result<(), SpecializedMeshPipelineError> {
        // This is the important part to tell bevy to render this material as a line between vertices
        descriptor.primitive.polygon_mode = PolygonMode::Line;
        Ok(())
    }
}

/// A list of lines with a start and end position
#[derive(Debug, Clone)]
pub struct LineList {
    pub lines: Vec<(Vec3, Vec3)>,
}

impl From<LineList> for Mesh {
    fn from(line: LineList) -> Self {
        let vertices: Vec<_> = line.lines.into_iter().flat_map(|(a, b)| [a, b]).collect();

        // This tells wgpu that the positions are list of lines
        // where every pair is a start and end point
        Mesh::new(PrimitiveTopology::LineList)
            // Add the vertices positions as an attribute
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, vertices)
    }
}

/// A list of points that will have a line drawn between each consecutive points
#[derive(Debug, Clone)]
pub struct LineStrip {
    pub points: Vec<Vec3>,
}

impl From<LineStrip> for Mesh {
    fn from(line: LineStrip) -> Self {
        // This tells wgpu that the positions are a list of points
        // where a line will be drawn between each consecutive point
        Mesh::new(PrimitiveTopology::LineStrip)
            // Add the point positions as an attribute
            .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, line.points)
    }
}
