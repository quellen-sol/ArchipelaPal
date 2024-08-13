use std::{
    io::{stdin, stdout, Write},
    sync::Arc,
    vec,
};

use anyhow::{anyhow, Result};
use ap_rs::client::ArchipelagoClient;
use clap::Parser;
use defs::{Config, FullGameState, GameMap, GoalOneShotData};
use processes::{
    game_playing_thread::spawn_game_playing_task, message_handler::spawn_ap_server_task,
};
use tokio::sync::oneshot;

mod defs;
mod processes;

#[derive(Parser)]
struct Args {
    #[clap(env)]
    password: String,

    #[clap(env)]
    slot_name: String,

    #[clap(env)]
    server_addr: Option<String>,
}

pub const GAME_NAME: &str = "APBot";
pub const ITEM_HANDLING: i32 = 0b111;

#[tokio::main]
async fn main() -> Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init();

    let args = Args::parse();

    let addr = args
        .server_addr
        .unwrap_or_else(|| get_user_input("Enter server ip and port:").unwrap());
    let mut client =
        ArchipelagoClient::with_data_package(&addr, Some(vec![GAME_NAME.into()])).await?;

    let _ = client
        .connect(
            GAME_NAME,
            &args.slot_name,
            Some(&args.password),
            Some(ITEM_HANDLING), // ?
            vec!["AP".into(), "Bot".into()],
        )
        .await?;

    log::info!("Connected");

    let this_game_data = client
        .data_package()
        .and_then(|dp| dp.games.get(GAME_NAME))
        .ok_or_else(|| anyhow!("Data package not preset for this game and slot???"))?;

    let info = client.room_info();
    log::info!("Seed: {}", info.seed_name);

    let game_state = FullGameState::from_file_or_default(&info.seed_name);

    // Correct the game state if it ended up being a default
    if game_state.seed_name.is_empty() {
        let loc_to_id = &this_game_data.location_name_to_id;
        let game_map = GameMap::new_from_data_package(loc_to_id);

        let mut map_lock = game_state.map.write().await;
        *map_lock = game_map;
    }

    let game_state = Arc::new(game_state);

    let (client_sender, client_receiver) = client.split();

    // TODO: Get this from output!
    let config = Config {
        min_wait_time: 1,
        max_wait_time: 1,
        num_goal: 10,
    };

    let (goal_tx, goal_rx) = oneshot::channel::<GoalOneShotData>();

    // Spawn server listen thread
    let server_handle =
        spawn_ap_server_task(game_state.clone(), client_receiver, config.clone(), goal_tx);
    let game_handle =
        spawn_game_playing_task(game_state.clone(), client_sender, config.clone(), goal_rx);

    server_handle.await.unwrap();
    game_handle.await.unwrap();

    Ok(())
}

fn get_user_input(prompt: &str) -> Result<String> {
    let mut buf = String::new();
    let sin = stdin();
    print!("{prompt}");
    stdout().flush()?;
    sin.read_line(&mut buf)?;

    Ok(buf.trim().to_string())
}
