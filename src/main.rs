use bevy::ecs::query;
use bevy::prelude::*;
use bevy::time::common_conditions::on_timer;
use bevy_inspector_egui::quick::WorldInspectorPlugin;
use bevy::window::PrimaryWindow;
use bevy_mod_reqwest::*;
use serde::Deserialize;
use std::time::Duration;

const BLOCK_SPEED: f32 = 10.0;

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
        .add_systems(Startup, spawn_camera)
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

pub fn spawn_camera(mut commands: Commands, window_query: Query<&Window, With<PrimaryWindow>>) {
    let window = window_query.get_single().unwrap();

    commands.spawn(Camera2dBundle {
        transform: Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 0.0),
        ..default()
    });
}

fn block_movement(mut enemy_query: Query<&mut Transform, With<Block>>, time: Res<Time>) {
    for mut transform in enemy_query.iter_mut() {
        let direction = Vec3::new(1.0, 0.0, 0.0);
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
    window_query: Query<&Window, With<PrimaryWindow>>,
    asset_server: Res<AssetServer>,

) {
    let window = window_query.get_single().unwrap();

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
                        commands.spawn((Block {
                            number,
                            gas_limit,
                            gas_used,
                        },
                        SpriteBundle {
                            transform: Transform::from_xyz(window.width() / 2.0, window.height() / 2.0, 0.0),
                            texture: asset_server.load("sprites/ball_blue_large.png"),
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

