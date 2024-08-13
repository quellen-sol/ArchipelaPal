use std::{sync::Arc, time::Duration};

use ap_rs::client::ArchipelagoClientSender;
use rand::{thread_rng, Rng};
use tokio::{sync::Mutex, task::JoinHandle};

use crate::defs::{Config, FullGameState};

pub fn spawn_game_playing_task(
    game_state: Arc<FullGameState>,
    sender: ArchipelagoClientSender,
    config: Config,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        let max_wait_time = config.max_wait_time;
        let min_wait_time = config.min_wait_time;
        loop {
            let wait_time = {
                let mut rng = thread_rng();
                rng.gen_range(min_wait_time..=max_wait_time)
                // `rng` must drop out of scope before entering back into async land
            };
            let duration = Duration::from_secs(wait_time.into());
            tokio::time::sleep(duration).await;
        }
    })
}
