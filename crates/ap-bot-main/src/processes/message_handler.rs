use anyhow::Result;
use ap_rs::{
    client::ArchipelagoClientReceiver,
    protocol::{ClientStatus, Hint, HintData, ServerMessage},
};
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};
use tokio::{sync::oneshot, task::JoinHandle};

use crate::defs::{
    game_state::FullGameState,
    lib::{GoalData, GoalOneShotData, OutputFileConfig},
};

pub fn spawn_ap_server_task(
    game_state: Arc<FullGameState>,
    mut client: ArchipelagoClientReceiver,
    config: OutputFileConfig,
    goal_tx: oneshot::Sender<GoalOneShotData>,
) -> JoinHandle<()> {
    println!("Now listening for AP server messages");
    tokio::spawn(async move {
        loop {
            let msg = client.recv().await;
            match msg {
                Ok(Some(msg)) => {
                    log::debug!("Got msg: {msg:?}");
                    match msg {
                        ServerMessage::ReceivedItems(items) => {
                            let mut player = game_state.player.write().await;
                            let last_idx = game_state.last_checked_idx.read().await;
                            if items.index == 0 {
                                // What we receive is the ENTIRE inventory when idx == 0
                                // Set the player's state and return
                                let new_player_inventory = items.items.into_iter().fold(
                                    HashMap::new(),
                                    |mut acc, curr| {
                                        if curr.item < 0 {
                                            return acc;
                                        }
                                        let id = curr.item as u32;
                                        let amt = acc.entry(id).or_insert(0);
                                        *amt += 1;

                                        acc
                                    },
                                );

                                player.inventory = new_player_inventory;
                            } else if items.index > *last_idx {
                                for item in items.items.iter() {
                                    let id = item.item;

                                    if id < 0 {
                                        // Special AP item. don't use
                                        continue;
                                    }
                                    let id = id as u32;

                                    // Append to inventory for now...
                                    let entry = player.inventory.entry(id).or_insert(0);
                                    *entry += 1;
                                }

                                // Drop read lock to get a write
                                drop(last_idx);

                                let mut last_idx_write = game_state.last_checked_idx.write().await;
                                *last_idx_write = items.index;
                            }
                            player.set_speed_modifier();

                            let player = player.downgrade();
                            // Quick goal check
                            let player_goaled = player
                                .inventory
                                .get(&0x010000)
                                .is_some_and(|v| *v >= config.num_goal);

                            if player_goaled {
                                log::info!("GOOOOAAALLLLL");
                                let data = GoalData {
                                    room_info: client.room_info().clone(),
                                };
                                goal_tx.send(data).unwrap();

                                // Need to more gracefully shutdown
                                log::info!("Server listening thread shutting down");
                                return;
                            }

                            game_state
                                .write_save_file()
                                .await
                                .inspect_err(|e| log::error!("Unable to write save file: {e}"))
                                .ok();
                        }
                        ServerMessage::Retrieved(retrieved) => {
                            for (key, val) in retrieved.keys.iter() {
                                if key.starts_with("_read_client_status") {
                                    if !key.ends_with(&game_state.slot_id.to_string()) {
                                        continue;
                                    }

                                    if val.is_null() {
                                        continue;
                                    }

                                    let status: Option<ClientStatus> = val
                                        .as_number()
                                        .and_then(|n| n.as_u64())
                                        .map(|n64| (n64 as u16).into());

                                    if let Some(ClientStatus::ClientGoal) = status {
                                        let data = GoalData {
                                            room_info: client.room_info().clone(),
                                        };
                                        goal_tx.send(data).unwrap();

                                        // Need to more gracefully shutdown
                                        log::info!("Server listening thread shutting down");
                                        return;
                                    }
                                } else if key.starts_with("_read_hints_") {
                                    if val.is_null() {
                                        continue;
                                    }

                                    let Some(hints) = val.as_array() else {
                                        log::error!("Hints not an array?");
                                        continue;
                                    };

                                    let hints_parsed = hints
                                        .iter()
                                        .filter_map(|v| {
                                            let parsed: Result<HintData> =
                                                serde_json::from_value::<Hint>(v.clone())
                                                    .map_err(Into::into)
                                                    .map(|v| v.into());
                                            let Ok(hint_data) = parsed else {
                                                log::error!("Failed to parse hint: {v}");
                                                return None;
                                            };

                                            if hint_data.found || !hint_data.is_important {
                                                return None;
                                            }

                                            Some(hint_data)
                                        })
                                        .collect::<HashSet<HintData>>();

                                    let mut source_hint_queue =
                                        game_state.source_hint_queue.write().await;
                                    *source_hint_queue = hints_parsed;
                                }
                            }
                        }
                        ServerMessage::PrintJSON(print_json) => {
                            if print_json.found.is_none() {
                                // Not a hint
                                continue;
                            }

                            let hint: HintData = print_json.into();

                            if hint.item.player == game_state.slot_id
                                && !hint.found
                                && hint.is_important
                            {
                                // This hint is an item that comes from us
                                let mut source_hint_queue =
                                    game_state.source_hint_queue.write().await;
                                source_hint_queue.insert(hint);
                            }
                        }
                        _ => {
                            // Supporting other packet types as needed
                            continue;
                        }
                    }
                }
                Ok(None) => {
                    log::warn!("Got None from AP server, continuing...");
                    continue;
                }
                Err(e) => {
                    // TODO: reconnect logic?
                    match e {
                        ap_rs::client::ArchipelagoError::FailedDeserialize(serde_err) => {
                            log::error!("{serde_err}");
                            continue;
                        }
                        _ => {
                            panic!("{e}");
                        }
                    }
                }
            }
        }
    })
}
