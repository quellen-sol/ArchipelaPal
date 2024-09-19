use std::{sync::Arc, time::Duration};

use ap_rs::{
    client::ArchipelagoClientSender,
    protocol::{Get, Permission},
};
use rand::{thread_rng, Rng};
use tokio::{
    sync::oneshot::{self, error::TryRecvError},
    task::JoinHandle,
};

use crate::defs::{
    game_state::FullGameState,
    lib::{GoalOneShotData, OutputFileConfig},
};

pub fn spawn_game_playing_task(
    game_state: Arc<FullGameState>,
    mut sender: ArchipelagoClientSender,
    config: OutputFileConfig,
    mut goal_rx: oneshot::Receiver<GoalOneShotData>,
) -> JoinHandle<()> {
    println!("Searching for items...");
    tokio::spawn(async move {
        let max_wait_time = config.max_wait_time;
        let min_wait_time = config.min_wait_time;
        loop {
            let wait_time = {
                // `rng` must drop out of scope before entering back into async land
                let mut rng = thread_rng();
                rng.gen_range(min_wait_time..=max_wait_time)
            };
            let player = game_state.player.read().await;
            let speed_modifier = &player.speed_modifier;
            let wait_time = ((wait_time as f32 / speed_modifier) * 1000.0) as u64;
            log::info!("waiting for {wait_time} ms");
            let duration = Duration::from_millis(wait_time);
            drop(player);
            tokio::time::sleep(duration).await;
            match goal_rx.try_recv() {
                Ok(data) => {
                    // We goaled!! Send packet to server
                    sender
                        .status_update(ap_rs::protocol::ClientStatus::ClientGoal)
                        .await
                        .unwrap();

                    sender.say("gg <3").await.ok();

                    // Check if we need to manually release
                    match data.room_info.permissions.release {
                        Permission::Enabled | Permission::Goal => {
                            log::info!("Releasing items...");
                            println!("Releasing items...");
                            sender.say("!release").await.unwrap();
                        }
                        _ => {
                            log::info!("I do not have to manually release!");
                        }
                    }

                    game_state
                        .write_save_file()
                        .await
                        .inspect_err(|e| log::error!("Error writing save file on goal: {e}"))
                        .ok();

                    // End the thread :)
                    log::info!("Shutting down gameplay thread");
                    return;
                }
                Err(e) => match e {
                    TryRecvError::Empty => {
                        // All good, we just haven't goaled yet.
                    }
                    TryRecvError::Closed => {
                        panic!("GOAL oneshot is poisoned!");
                    }
                },
            };

            // A write lock is grabbed here, and ofc released after finishing
            let location_checked = game_state.tick_game_state().await;

            match location_checked {
                None => {
                    // BK'd!
                    log::warn!("I'm BK'd!!!");
                    println!("Currently in BK mode!");
                }
                Some(loc_id) => {
                    // Found an item!
                    println!("Checked location ID: {loc_id}");
                    let loc_id = loc_id as i32;
                    match sender.location_checks(vec![loc_id]).await {
                        Ok(_) => {
                            // Remove from hint queue
                            let mut source_hint_queue = game_state.source_hint_queue.write().await;
                            source_hint_queue.retain(|hint| hint.item.location != loc_id);
                        }
                        Err(e) => {
                            log::error!("{e}");
                        }
                    };
                }
            }

            let hint_get_key = game_state.make_hints_get_key(game_state.slot_id);

            sender
                .send(ap_rs::protocol::ClientMessage::Get(Get {
                    keys: vec![hint_get_key],
                }))
                .await
                .ok();
        }
    })
}
