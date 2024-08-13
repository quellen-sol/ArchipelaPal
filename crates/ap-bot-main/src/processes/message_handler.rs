use ap_rs::client::ArchipelagoClientReceiver;
use std::sync::Arc;
use tokio::{sync::mpsc::Sender, task::JoinHandle};

use super::game_state_handler::StateMessage;

pub fn spawn_ap_server_task(
    sender: Arc<Sender<StateMessage>>,
    mut client: ArchipelagoClientReceiver,
) -> JoinHandle<()> {
    tokio::spawn(async move {
        loop {
            let msg = client.recv().await;
            match msg {
                Ok(Some(msg)) => {
                    log::info!("{msg:?}");
                    sender.send(StateMessage::ServerMessage(msg)).await.unwrap();
                }
                Ok(None) => continue,
                Err(e) => {
                    // TODO: reconnect logic?
                    panic!("{e}");
                }
            }
        }
    })
}
