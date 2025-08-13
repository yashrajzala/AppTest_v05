#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod services {
    pub mod mqtt;
    pub mod storage;
}

use services::mqtt::greenhouse_sensor::{
    subscriber::run_debug_subscriber,
    aggregator::{run_rolling_avg, NodeAvg},
    greenhouse_aggregator::{run_greenhouse_avg, GhAvg},
};
use services::storage::sqlite::run_storage;

use tokio::sync::mpsc;

const DB_PATH: &str = "../data/app.db"; // keep DB outside src-tauri

#[tokio::main]
async fn main() {
    tauri::Builder::default()
        .setup(|app| {
            // Stage 1: decoded samples from MQTT subscriber
            let (tx_decoded, rx_decoded) = mpsc::channel(256);

            // Stage 2 outputs: per-node 60s averages
            let (tx_nodeavg_for_gh, rx_nodeavg_for_gh) = mpsc::channel::<NodeAvg>(128);
            let (tx_nodeavg_for_db, rx_nodeavg_for_db) = mpsc::channel::<NodeAvg>(128);

            // Stage 3 outputs: greenhouse 60s averages
            let (tx_ghavg_for_db, rx_ghavg_for_db) = mpsc::channel::<GhAvg>(64);
            let (tx_ghavg_for_ui, mut rx_ghavg_for_ui) = mpsc::channel::<GhAvg>(64);

            // DB writer task
            tauri::async_runtime::spawn(async move {
                run_storage(DB_PATH, rx_nodeavg_for_db, rx_ghavg_for_db).await;
            });

            // Greenhouse aggregator (NodeAvg -> GhAvg -> DB & UI)
            let tx_ghavg_for_db_clone = tx_ghavg_for_db.clone();
            let tx_ghavg_for_ui_clone = tx_ghavg_for_ui.clone();
            tauri::async_runtime::spawn(async move {
                run_greenhouse_avg(rx_nodeavg_for_gh, tx_ghavg_for_db_clone, tx_ghavg_for_ui_clone).await;
            });

            // Node rolling averages (Decoded -> NodeAvg for GH & DB)
            let tx_nodeavg_for_gh_clone = tx_nodeavg_for_gh.clone();
            let tx_nodeavg_for_db_clone = tx_nodeavg_for_db.clone();
            tauri::async_runtime::spawn(async move {
                run_rolling_avg(rx_decoded, tx_nodeavg_for_db_clone, tx_nodeavg_for_gh_clone).await;
            });

            // MQTT subscriber (hot path)
            tauri::async_runtime::spawn(async move {
                run_debug_subscriber(tx_decoded).await;
            });

            // UI emitter: forward full GhAvg to frontend ("gh_avg" events)
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                use tauri::Emitter;
                while let Some(ga) = rx_ghavg_for_ui.recv().await {
                    // GhAvg already derives Serialize + Clone
                    let _ = app_handle.emit("gh_avg", ga);
                }
            });

            Ok(())
        })
        .run(tauri::generate_context!())
        .expect("error while running Tauri application");
}
