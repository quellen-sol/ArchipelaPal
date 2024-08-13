use ap_rs::{client::ArchipelagoClientReceiver, protocol::ServerMessage};
use std::sync::Arc;
use tokio::{sync::oneshot, task::JoinHandle};

use crate::defs::{Config, FullGameState, GoalData, GoalOneShotData};

pub fn spawn_ap_server_task(
    game_state: Arc<FullGameState>,
    mut client: ArchipelagoClientReceiver,
    config: Config,
    goal_tx: oneshot::Sender<GoalOneShotData>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            let msg = client.recv().await;
            match msg {
                Ok(Some(msg)) => {
                    match msg {
                        ServerMessage::ReceivedItems(items) => {
                            log::debug!("Getting player write lock");
                            let mut player = game_state.player.write().await;
                            log::debug!("Got player write lock");
                            for item in items.items.iter() {
                                let id = item.item;
                                if id < 0 {
                                    // Special AP item. don't use
                                    continue;
                                }
                                let id = id as u32;
                                let entry = player.inventory.entry(id).or_insert(0);
                                *entry += 1;
                            }

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
                                break;
                            }
                        }

                        _ => {
                            // Supporting other packet types in the future
                            continue;
                        }
                    }
                }
                Ok(None) => continue,
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
