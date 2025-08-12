#[derive(Clone, Copy)]
pub struct MqttAuth<'a> {
    pub host: &'a str,
    pub port: u16,
    pub client_id_prefix: &'a str,
    pub username: &'a str,
    pub password: &'a str,
    pub keep_alive_secs: u16,
}

pub const fn mqtt_auth() -> MqttAuth<'static> {
    MqttAuth {
        host: "192.168.20.1",
        port: 1883,
        client_id_prefix: "tauri-greenhouse",
        username: "cresla",
        password: "cresla123.",
        keep_alive_secs: 30,
    }
}
