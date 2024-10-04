use std::{
    fs,
    io::{stdin, stdout, Write},
    sync::Arc,
    vec,
};

use anyhow::{anyhow, bail, Context, Result};
use ap_rs::{client::ArchipelagoClient, protocol::Get};
use clap::Parser;
use defs::{
    game_state::{FullGameState, GameMap},
    lib::{GoalOneShotData, OutputFileConfig, SAVE_FILE_DIRECTORY},
    user_settings::UserSettings,
};
use processes::{
    game_playing_thread::spawn_game_playing_task, message_handler::spawn_ap_server_task,
};
use tokio::sync::oneshot;

mod defs;
mod processes;

#[derive(Parser)]
struct Args {
    #[clap(long, short, env)]
    slot_name: Option<String>,

    #[clap(long, short = 'a', env)]
    server_addr: Option<String>,

    #[clap(long, short, env)]
    password: Option<String>,

    #[clap(long)]
    skip_start_confirmation: bool,
}

pub const GAME_NAME: &str = "ArchipelaPal";
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
    let mut user_settings = UserSettings::load_or_default();

    let slot_name = args
        .slot_name
        .or_else(|| {
            let last_slot_name = &user_settings.last_used_slot;
            let prompt = match last_slot_name {
                Some(name) => format!("Enter slot name (Press Enter for last used: \"{name}\"):"),
                None => "Enter slot name:".to_string(),
            };
            let user_input = get_user_input(&prompt).unwrap();
            match (user_input, last_slot_name) {
                (input, _) if input.is_empty() => last_slot_name.clone(),
                (input, _) => Some(input),
            }
        })
        .ok_or_else(|| anyhow!("Slot name cannot be empty!"))?;

    user_settings.last_used_slot = Some(slot_name.clone());

    let addr = args
        .server_addr
        .or_else(|| {
            let last_server_addr = &user_settings.last_used_address;
            let prompt = match last_server_addr {
                Some(addr) => {
                    format!("Enter server address (Press Enter for last used: \"{addr}\"):")
                }
                None => "Enter server address:".to_string(),
            };
            let user_input = get_user_input(&prompt).unwrap();
            match (user_input, last_server_addr) {
                (input, _) if input.is_empty() => last_server_addr.clone(),
                (input, _) => Some(input),
            }
        })
        .ok_or_else(|| anyhow!("Server address cannot be empty!"))?;

    user_settings.last_used_address = Some(addr.clone());

    let password = args
        .password
        .unwrap_or_else(|| get_user_input("Enter server password (Press Enter if none):").unwrap());

    user_settings
        .save()
        .context("Could not save user settings")?;

    let mut client = ArchipelagoClient::new(&addr).await?;

    let connected_packet = client
        .connect(
            GAME_NAME,
            &slot_name,
            Some(&password),
            Some(ITEM_HANDLING), // ?
            vec!["AP".into(), "Bot".into()],
            true,
        )
        .await?;

    let config = serde_json::from_value::<OutputFileConfig>(connected_packet.slot_data)
        .context("Could not parse slot_data??")?;

    log::debug!("Config: {config:?}");

    log::info!("Connected");

    let info = client.room_info();
    log::info!("Seed: {}", info.seed_name);

    // Make 'Saves' directory if it doesn't exist
    fs::create_dir_all(SAVE_FILE_DIRECTORY).context("Could not create 'Saves' directory")?;

    let mut game_state = FullGameState::from_file_or_default(&info.seed_name);

    let slot_id = connected_packet.slot;
    let team = connected_packet.team;

    // Correct the game state if it ended up being a default
    if game_state.seed_name.is_empty() {
        let game_map = GameMap::new_from_config(&config);

        let mut map_lock = game_state.map.write().await;
        *map_lock = game_map;
        drop(map_lock);

        // GAME STATE FIRST TIME CREATION
        game_state.seed_name = info.seed_name.clone();
        game_state.team = team;
        game_state.slot_id = slot_id;
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
            keys: vec![
                format!("_read_client_status_{team}_{slot_id}"),
                game_state.make_hints_get_key(slot_id),
            ],
        }))
        .await
        .context("Failed to get my status!")?;

    client_sender
        .send(ap_rs::protocol::ClientMessage::Sync)
        .await
        .context("Could not send sync packet!")?;

    if !args.skip_start_confirmation {
        // Prompt user to start game "press enter to start"
        let start_prompt = format!("Press Enter to start {GAME_NAME} for slot {slot_name}...");
        get_user_input(&start_prompt)?;
    }

    let game_handle =
        spawn_game_playing_task(game_state.clone(), client_sender, config.clone(), goal_rx);

    let (sh_joined, gh_joined) = tokio::join!(server_handle, game_handle);

    sh_joined?;
    gh_joined?;

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
