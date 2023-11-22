use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy_mod_reqwest::*;
use bevy_panorbit_camera::*;
use serde::Deserialize;
use std::time::Duration;

const BLOCK_SPEED: f32 = 0.2;

#[derive(Deserialize)]
struct BlockResponse {
    #[serde(rename = "gasLimit")]
    gas_limit: String,
    #[serde(rename = "gasUsed")]
    gas_used: String,
    number: String,
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
        .register_type::<Block>()
        .add_systems(Startup, setup)
        .add_plugins(PanOrbitCameraPlugin)
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
    commands.spawn(PbrBundle {
        mesh: meshes.add(shape::Circle::new(400.0).into()),
        material: materials.add(Color::WHITE.into()),
        transform: Transform::from_rotation(Quat::from_rotation_x(-std::f32::consts::FRAC_PI_2)),
        ..default()
    });
    // light
    commands.spawn(PointLightBundle {
        point_light: PointLight {
            intensity: 1500.0,
            shadows_enabled: true,
            ..default()
        },
        transform: Transform::from_xyz(4.0, 8.0, 4.0),
        ..default()
    });
    // camera
    commands.spawn((Camera3dBundle {
        transform: Transform::from_xyz(0.8, 1.1, 4.0).looking_at(Vec3::ZERO, Vec3::Y),
        ..default()
    }, PanOrbitCamera::default(),));
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

        if let Ok(number) = u64::from_str_radix(&a.result.number[2..], 16) {
            if let Ok(gas_limit) = u64::from_str_radix(&a.result.gas_limit[2..], 16) {
                if let Ok(gas_used) = u64::from_str_radix(&a.result.gas_used[2..], 16) {

                    let block_exists = query.iter().any(|block| block.number == number);

                    if block_exists {
                        println!("Already have this block");
                    } else {
                        println!("spawning new block {}", number);
                        let ratio = gas_used as f32 / gas_limit as f32;

                        let height = 1.0 * ratio;
                        let center_translation = 0.5 * height;

                        println!("ratio {}", ratio);
                        commands.spawn((Block {
                            number,
                            gas_limit,
                            gas_used,
                        },
                            // cube
                        PbrBundle {
                            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                            material: materials.add(Color::rgb_u8(124, 144, 255).with_a(0.5).into()),
                            transform: Transform::from_xyz(0.0, 0.5, 0.0),
                            ..default()
                        },
                        ));
                        commands.spawn((Block {
                            number,
                            gas_limit,
                            gas_used,
                        },
                            // cube
                        PbrBundle {
                            mesh: meshes.add(Mesh::from(shape::Cube { size: 1.0 })),
                            material: materials.add(Color::rgb_u8(124, 144, 255).into()),
                            transform: Transform::from_xyz(0.0, center_translation, 0.0).with_scale(Vec3::new(0.99, ratio, 0.99)),
                            ..default()
                        },
                        ));

                    }
                    

                }
            }
        }
    


        // Done with this entity
        commands.entity(e).despawn_recursive();
    }
}

