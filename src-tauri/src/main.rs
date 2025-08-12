// Prevents additional console window on Windows in release, DO NOT REMOVE!!
#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod services {
    pub mod mqtt;
}
use services::mqtt::greenhouse_sensor::{subscriber::run_debug_subscriber, aggregator::run_rolling_avg};
use tokio::sync::mpsc;

fn main() {
    tauri::Builder::default()
        .setup(|_app| {
            // Channel between subscriber (producer) and aggregator (consumer).
            // Capacity sized for burst safety while staying small.
            let (tx, rx) = mpsc::channel(256);

            // Aggregator prints once per 60s.
            tauri::async_runtime::spawn(async move {
                run_rolling_avg(rx).await;
            });

            // MQTT subscriber feeds decoded samples; never blocks on send.
            tauri::async_runtime::spawn(async move {
                run_debug_subscriber(tx).await;
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}
