use rumqttc::{AsyncClient, EventLoop, MqttOptions};
use std::time::Duration;
use super::config::MqttAuth;

/// Very small internal queues to avoid memory bloat.
pub fn new_client(client_id_suffix: &str, auth: MqttAuth) -> (AsyncClient, EventLoop) {
    let mut opts = MqttOptions::new(
        format!("{}-{}", auth.client_id_prefix, client_id_suffix),
        auth.host,
        auth.port,
    );
    opts.set_credentials(auth.username, auth.password);
    opts.set_keep_alive(Duration::from_secs(auth.keep_alive_secs as u64));
    AsyncClient::new(opts, 10)
}
