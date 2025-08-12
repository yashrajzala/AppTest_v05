//! Resilient, non-blocking MQTT subscriber for greenhouse sensor data.
//! - Sends decoded samples to the rolling-average aggregator via mpsc.
//! - No raw prints here (keeps terminal output to 60s AVG only).

use rumqttc::{Event, Packet, QoS};
use std::time::Duration;
use tokio::{sync::mpsc, time::sleep};

use crate::services::mqtt::config::mqtt_auth;
use crate::services::mqtt::core::new_client;
use super::decoder::{decode_payload, Decoded};

/// Public entry: provide a Sender so we never block on the hot path.
/// We use `try_send` to avoid backpressure stalls; if full, we drop a sample.
pub async fn run_debug_subscriber(tx: mpsc::Sender<Decoded>) {
    let auth = mqtt_auth();
    let topic = "greenhouse/+/node/+/data";

    let mut backoff_ms: u64 = 250;

    loop {
        let (client, mut eventloop) = new_client("sensor-subscriber", auth);

        if let Err(e) = client.subscribe(topic, QoS::AtLeastOnce).await {
            eprintln!("[MQTT] subscribe error: {e}");
            sleep(Duration::from_millis(backoff_ms)).await;
            backoff_ms = (backoff_ms * 2).min(10_000);
            continue;
        }

        println!("[MQTT] Subscribed: '{topic}'");

        loop {
            match eventloop.poll().await {
                Ok(Event::Incoming(Packet::Publish(p))) => {
                    if let Some(decoded) = decode_payload(&p.payload) {
                        // Non-blocking send; drop if channel is full to keep MQTT loop hot.
                        let _ = tx.try_send(decoded);
                    } else {
                        eprintln!("[DATA] decode skipped: malformed payload ({} bytes)", p.payload.len());
                    }
                }
                Ok(Event::Incoming(_)) => {}
                Ok(Event::Outgoing(_)) => {}
                Err(e) => {
                    eprintln!("[MQTT] eventloop error: {e}");
                    break; // reconnect with backoff
                }
            }
        }

        sleep(Duration::from_millis(backoff_ms)).await;
        backoff_ms = (backoff_ms * 2).min(10_000);
    }
}
