use std::{
    fs,
    io::{stdin, stdout, Write},
    path::PathBuf,
    sync::Arc,
    vec,
};

use anyhow::{anyhow, bail, Context, Result};
use ap_rs::{client::ArchipelagoClient, protocol::Get};
use clap::Parser;
use defs::{FullGameState, GameMap, GoalOneShotData, OutputFileConfig};
use processes::{
    game_playing_thread::spawn_game_playing_task, message_handler::spawn_ap_server_task,
};
use rfd::FileDialog;
use tokio::sync::oneshot;

mod defs;
mod processes;

#[derive(Parser)]
struct Args {
    #[clap()]
    output_file: Option<PathBuf>,

    #[clap(long, short, env)]
    slot_name: Option<String>,

    #[clap(long, short = 'a', env)]
    server_addr: Option<String>,

    #[clap(long, short, env)]
    password: Option<String>,
}

pub const GAME_NAME: &str = "APBot";
pub const ITEM_HANDLING: i32 = 0b111;

#[tokio::main]
async fn main() -> Result<()> {
    let main_result = outer_main().await;

    match main_result {
        Ok(_) => Ok(()),
        Err(e) => {
            log::error!("{e}");
            println!("{e}");
            get_user_input("Press Enter to exit...")?;
            bail!(e)
        }
    }
}

// This is our primary `main` function, but to allow for easy error handling and logging, we have
// the actual main function call this one.
async fn outer_main() -> Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init();

    let args = Args::parse();

    let addr = args
        .server_addr
        .unwrap_or_else(|| get_user_input("Enter server ip and port:").unwrap());

    let password = args
        .password
        .unwrap_or_else(|| get_user_input("Enter server password (Press Enter if none):").unwrap());

    let slot_name = args
        .slot_name
        .unwrap_or_else(|| get_user_input("Enter slot name:").unwrap());

    let mut client =
        ArchipelagoClient::with_data_package(&addr, Some(vec![GAME_NAME.into()])).await?;

    let config = load_output_file(args.output_file)?;

    let connected_packet = client
        .connect(
            GAME_NAME,
            &slot_name,
            Some(&password),
            Some(ITEM_HANDLING), // ?
            vec!["AP".into(), "Bot".into()],
        )
        .await?;

    log::info!("Connected");

    let team = connected_packet
        .players
        .iter()
        .find_map(|p| {
            if p.name == config.slot_name {
                Some(p.team)
            } else {
                None
            }
        })
        .ok_or_else(|| anyhow!("No player in server list with name {}", config.slot_name))?;

    let this_game_data = client
        .data_package()
        .and_then(|dp| dp.games.get(GAME_NAME))
        .ok_or_else(|| anyhow!("Data package not preset for this game and slot???"))?;

    let info = client.room_info();
    log::info!("Seed: {}", info.seed_name);

    let mut game_state = FullGameState::from_file_or_default(&info.seed_name);

    // Correct the game state if it ended up being a default
    if game_state.seed_name.is_empty() {
        let loc_to_id = &this_game_data.location_name_to_id;
        let game_map = GameMap::new_from_data_package(loc_to_id);

        let mut map_lock = game_state.map.write().await;
        *map_lock = game_map;
        drop(map_lock);

        game_state.seed_name = info.seed_name.clone();
        game_state.team = team;
    }

    let game_state = Arc::new(game_state);

    let (mut client_sender, client_receiver) = client.split();

    let (goal_tx, goal_rx) = oneshot::channel::<GoalOneShotData>();

    // Spawn server listen thread
    let server_handle =
        spawn_ap_server_task(game_state.clone(), client_receiver, config.clone(), goal_tx);

    // Task started, slight delay, then send syncing packets
    client_sender
        .send(ap_rs::protocol::ClientMessage::Get(Get {
            keys: vec![format!("client_status_{team}_{}", config.slot_name)],
        }))
        .await
        .context("Failed to get my status!")?;

    client_sender
        .send(ap_rs::protocol::ClientMessage::Sync)
        .await
        .context("Could not send sync packet!")?;

    let game_handle =
        spawn_game_playing_task(game_state.clone(), client_sender, config.clone(), goal_rx);

    let (sh_joined, gh_joined) = tokio::join!(server_handle, game_handle);

    sh_joined.context("Server thread panicked")?;
    gh_joined.context("Game thread panicked")?;

    Ok(())
}

fn load_output_file(path: Option<PathBuf>) -> Result<OutputFileConfig> {
    let path = path
        .or_else(|| {
            FileDialog::new()
                .set_title("Select the output file for this seed")
                .add_filter("APBot Output File", &["json"])
                .pick_file()
        })
        .ok_or_else(|| anyhow!("Could not find an output file for this seed!"))?;

    Ok(fs::read_to_string(&path)
        .and_then(|file_s| Ok(serde_json::from_str::<OutputFileConfig>(&file_s)?))?)
}

fn get_user_input(prompt: &str) -> Result<String> {
    let mut buf = String::new();
    let sin = stdin();
    print!("{prompt}");
    stdout().flush()?;
    sin.read_line(&mut buf)?;

    Ok(buf.trim().to_string())
}
