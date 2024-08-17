use std::{sync::Arc, time::Duration};

use ap_rs::{client::ArchipelagoClientSender, protocol::Permission};
use rand::{thread_rng, Rng};
use tokio::{
    sync::oneshot::{self, error::TryRecvError},
    task::JoinHandle,
};

use crate::defs::{Config, FullGameState, GoalOneShotData};

pub fn spawn_game_playing_task(
    game_state: Arc<FullGameState>,
    mut sender: ArchipelagoClientSender,
    config: Config,
    mut goal_rx: oneshot::Receiver<GoalOneShotData>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let max_wait_time = config.max_wait_time;
        let min_wait_time = config.min_wait_time;
        loop {
            let wait_time = {
                // `rng` must drop out of scope before entering back into async land
                let mut rng = thread_rng();
                rng.gen_range(min_wait_time..=max_wait_time)
            };
            let duration = Duration::from_secs(wait_time.into());
            tokio::time::sleep(duration).await;
            match goal_rx.try_recv() {
                Ok(data) => {
                    // We goaled!! Send packet to server
                    sender
                        .status_update(ap_rs::protocol::ClientStatus::ClientGoal)
                        .await
                        .unwrap();

                    sender.say("gg <3").await.unwrap();

                    // Check if we need to manually release
                    match data.room_info.permissions.release {
                        Permission::Enabled | Permission::Goal => {
                            log::info!("Releasing items...");
                            sender.say("!release").await.unwrap();
                        }
                        _ => {
                            log::info!("I do not have to manually release!");
                        }
                    }

                    // End the thread :)
                    log::info!("Shutting down gameplay thread");
                    return;
                }
                Err(e) => match e {
                    TryRecvError::Closed => {
                        panic!("GOAL oneshot is poisoned!");
                    }
                    TryRecvError::Empty => {
                        // All good, we just haven't goaled yet.
                    }
                },
            };

            // A write lock is grabbed here, and ofc released after finishing
            let location_checked = game_state.tick_game_state().await;

            match location_checked {
                None => {
                    // BKd!
                    log::warn!("I'm BKd!!!");
                    // log::debug!("{game_state:?}");
                }
                Some(loc_id) => {
                    // Send a checked location packet!!! 🚀
                    match sender.location_checks(vec![loc_id as i32]).await {
                        Ok(_) => {}
                        Err(e) => {
                            log::error!("{e}");
                        }
                    };
                }
            }
        }
    })
}
