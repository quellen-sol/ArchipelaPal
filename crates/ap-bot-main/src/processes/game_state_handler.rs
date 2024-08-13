use std::sync::Arc;

use ap_rs::protocol::ServerMessage;
use tokio::{
    sync::{mpsc::Receiver, Mutex},
    task::JoinHandle,
};

use crate::defs::FullGameState;

pub enum StateMessage {
    ServerMessage(ServerMessage),
}

pub fn spawn_game_state_task(
    mut state_receiver: Receiver<StateMessage>,
    game_state: Arc<FullGameState>,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            let msg = state_receiver.recv().await.unwrap();

            // match msg {

            // }
        }
    })
}
