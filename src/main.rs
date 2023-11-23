use bevy::{
    core_pipeline::{
        bloom::{BloomCompositeMode, BloomSettings},
        tonemapping::Tonemapping,
    },
    render::camera::ScalingMode,
    prelude::*,
};

use bevy::time::common_conditions::on_timer;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_mod_reqwest::*;
use bevy_panorbit_camera::*;
use serde::Deserialize;
use std::time::Duration;
use bevy_mod_picking::{DefaultPickingPlugins, PickableBundle};

const BLOCK_SPEED: f32 = 0.2;
const tx_spacing: f32 = 0.05;


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
        .insert_resource(ClearColor(Color::rgb(0.0, 0.0, 0.0)))
        .register_type::<Block>()
        .add_systems(Startup, setup)
        .add_plugins(PanOrbitCameraPlugin)
        .add_plugins(DefaultPickingPlugins)
        .add_plugins((
            DefaultPlugins,
            WorldInspectorPlugin::default(),
            ReqwestPlugin,
        ))
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
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    // circular base
    // commands.spawn(PbrBundle {
    //     mesh: meshes.add(shape::Circle::new(400.0).into()),
    //     material: materials.add(Color::GRAY.into()),
    //     transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
    //     ..default()
    // });
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 200.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 6.0, 4.0),
        ..default()
    });


    commands.spawn((
        Camera3dBundle {
            projection: OrthographicProjection {
                near: 0.0,
                far: 500.0,
                scale: 8.0,
                scaling_mode: ScalingMode::FixedVertical(0.8),
                ..default()
            }
            .into(),
            camera: Camera {
                hdr: true, // 1. HDR is required for bloom

                ..default()
            },
            tonemapping: Tonemapping::TonyMcMapface, // 2. Using a tonemapper that desaturates to white is recommended
            transform: Transform::from_xyz(0.8, 1.1, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
            ..default()
        },
        BloomSettings{
            composite_mode: BloomCompositeMode::Additive, // 3. Add the bloom to the scene
            ..Default::default()
        }, // 3. Enable bloom for the camera
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

    let req = reqwest.0.post(url).json(&serde_json::json!({
        "jsonrpc": "2.0",
        "method": "eth_getBlockByNumber",
        "params": ["latest", true],
        "id": 1,
    })).build().unwrap();
    let req = ReqwestRequest::new(req);
    commands.spawn(req);
}

fn handle_responses(
    mut commands: Commands, 
    results: Query<(Entity, &ReqwestBytesResult)>, 
    query: Query<&Block>,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {

    for (e, res) in results.iter() {
        let a: Response = serde_json::from_slice(res.as_ref().unwrap()).unwrap();



        // let transactions: Vec<Transaction> = a.result.transactions.clone().into_iter().map(|t| {
        //     let block_number = u64::from_str_radix(&t.block_number[2..], 16).unwrap();
        //     let gas = u64::from_str_radix(&t.gas[2..], 16).unwrap();
        //     let index = u64::from_str_radix(&t.index[2..], 16).unwrap();
        //     Transaction {
        //         block_number,
        //         gas,
        //         index,
        //     }
        // });
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
                        commands.spawn(Block {
                            number,
                            gas_limit,
                            gas_used,
                        })
                        .insert(PbrBundle {
                            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                            material: materials.add(Color::rgb_u8(124, 144, 255).with_a(1.0).into()),
                            transform: Transform::from_xyz(0.0, 0.5, 0.0),
                            ..default()
                        })
                        .insert(PickableBundle::default())
                        .with_children(|parent| {
                            parent.spawn(PbrBundle {
                            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                            material: materials.add(StandardMaterial {
                                emissive: Color::rgb_u8(124, 144, 255), // 4. Put something bright in a dark environment to see the effect
                                ..default()
                            }),
                            // transform: Transform::from_xyz(0.0, center_translation, 0.0).with_scale(Vec3::new(0.99, ratio, 0.99)),
                            transform: Transform::from_xyz(0.0, center_translation, 0.0).with_scale(Vec3::new(1.01, ratio, 1.01)),
                            ..default()
                            });

                            let mut offset = 1.0 / 2.0 - tx_spacing;
                            // spawn cubes for each transaction spaced vertically
                            for (i, t) in a.result.transactions.iter().enumerate() {
                                let gas = u64::from_str_radix(&t.gas[2..], 16).unwrap();
                                let index = u64::from_str_radix(&t.index[2..], 16).unwrap();
                                let tx_ratio = gas as f32 / gas_limit as f32;
                                let tx_translation = offset - (tx_ratio / 2.0);
                                parent.spawn(PbrBundle {
                                    mesh: meshes.add(Mesh::from(shape::Cube { size: ratio })),
                                    material: materials.add(StandardMaterial {
                                        emissive: Color::rgb_u8(124, 144, 255), // 4. Put something bright in a dark environment to see the effect
                                        ..default()
                                    }),
                                    transform: Transform::from_xyz(0.0, tx_translation, 0.0).with_scale(tx_ratio * Vec3::new(1.0, 1.0, 1.0)),
                                    ..default()
                                })
                                .insert(PickableBundle::default());
                                offset -= tx_ratio + tx_spacing;
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

