use std::{fs, time::Instant};

use archipelago_rs::{client::ArchipelagoClient, protocol::ServerMessage};
use clap::Parser;

#[derive(Parser)]
struct Args {
    #[clap(long, short, env)]
    pub url: String,

    #[clap(long, short, env)]
    pub slot_name: String,

    #[clap(long, short, env)]
    pub game: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();
    env_logger::init();
    let args = Args::parse();

    let mut client = ArchipelagoClient::new(&args.url).await?;

    client
        .connect(
            &args.game,
            &args.slot_name,
            None,
            Some(0b111),
            vec!["Tracker".into()],
            true,
        )
        .await?;

    let mut elapsed_points = Vec::new();
    let mut current_checks = 0;
    let mut lowest_time = 0;
    let mut running_avg_total = 0;
    let mut average_time = 0.0;
    let mut highest_time = 0;
    let mut last_check_time = Instant::now();

    loop {
        let Ok(Some(p)) = client.recv().await else {
            continue;
        };

        match p {
            ServerMessage::ReceivedItems(items) => {
                if items.index <= 0 {
                    continue;
                }
                let elapsed = last_check_time.elapsed().as_secs();
                elapsed_points.push(elapsed.to_string());
                log::warn!("Elapsed: {elapsed}");
                if current_checks == 0 {
                    lowest_time = elapsed;
                    highest_time = elapsed;
                    average_time = elapsed as f64;
                    running_avg_total = elapsed;
                } else {
                    lowest_time = lowest_time.min(elapsed);
                    highest_time = highest_time.max(elapsed);
                    let avg_factor = ((running_avg_total as f64 + elapsed as f64)
                        * current_checks as f64)
                        / ((current_checks as f64 + 1.0) * running_avg_total as f64);
                    average_time *= avg_factor;
                    running_avg_total += elapsed;
                }

                current_checks += 1;
                last_check_time = Instant::now();

                fs::write("elapsed_points.csv", elapsed_points.join("\n"))?;

                log::warn!("Checks: {current_checks}, Lowest: {lowest_time}, Highest: {highest_time}, Average: {average_time:.2}");
            }
            _ => continue,
        }
    }
}
