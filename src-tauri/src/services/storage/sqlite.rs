//! Async, durable SSD storage using rusqlite.
//! - 5-table schema (greenhouse_id, sensor_type, greenhouse_average, node_name, node_values).
//! - FK ON, WAL, NORMAL sync.
//! - Per-insert error handling: bad rows are logged and skipped (no crash).
//! - 2-decimal rounding on floats for consistent storage.
//! - Prints the absolute DB path on init so you can open it in a viewer.

use std::{fs, path::PathBuf, time::{SystemTime, UNIX_EPOCH}};
use tokio::{sync::mpsc, time::{interval, Duration}};
use rusqlite::{Connection, params};

use crate::services::mqtt::greenhouse_sensor::aggregator::NodeAvg;
use crate::services::mqtt::greenhouse_sensor::greenhouse_aggregator::GhAvg;

#[inline]
fn now_ms() -> i64 {
    SystemTime::now().duration_since(UNIX_EPOCH).unwrap().as_millis() as i64
}
#[inline]
fn r2(v: Option<f32>) -> Option<f64> { v.map(|x| ((x as f64) * 100.0).round() / 100.0) }

fn absolute_path(db_path: &str) -> PathBuf {
    let cwd = std::env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    cwd.join(db_path)
}

fn open_and_init(path: &str) -> rusqlite::Result<Connection> {
    // ensure directory exists
    if let Some(dir) = std::path::Path::new(path).parent() {
        if !dir.as_os_str().is_empty() { let _ = fs::create_dir_all(dir); }
    }
    let conn = Connection::open(path)?;
    conn.pragma_update(None, "foreign_keys", &"ON")?;
    conn.pragma_update(None, "journal_mode", &"WAL")?;
    conn.pragma_update(None, "synchronous", &"NORMAL")?;

    // NOTE: renamed "values" -> "node_values" (avoid SQL keyword)
    conn.execute_batch(r#"
      CREATE TABLE IF NOT EXISTS greenhouse_id (
        id INTEGER PRIMARY KEY
      );
      CREATE TABLE IF NOT EXISTS sensor_type (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        key TEXT NOT NULL UNIQUE,
        unit TEXT NOT NULL
      );
      CREATE TABLE IF NOT EXISTS greenhouse_average (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        ts_ms INTEGER NOT NULL,
        greenhouse_id INTEGER NOT NULL,
        sensor_type_id INTEGER NOT NULL,
        value REAL,
        nodes INTEGER NOT NULL,
        agg TEXT NOT NULL,
        window_sec INTEGER NOT NULL,
        UNIQUE(ts_ms, greenhouse_id, sensor_type_id, agg),
        FOREIGN KEY (greenhouse_id) REFERENCES greenhouse_id(id) ON DELETE CASCADE,
        FOREIGN KEY (sensor_type_id) REFERENCES sensor_type(id) ON DELETE RESTRICT
      );
      CREATE TABLE IF NOT EXISTS node_name (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        greenhouse_id INTEGER NOT NULL,
        node_id INTEGER NOT NULL,
        label TEXT NOT NULL,
        UNIQUE(greenhouse_id, node_id),
        FOREIGN KEY (greenhouse_id) REFERENCES greenhouse_id(id) ON DELETE CASCADE
      );
      CREATE TABLE IF NOT EXISTS node_values (
        id INTEGER PRIMARY KEY AUTOINCREMENT,
        ts_ms INTEGER NOT NULL,
        node_id INTEGER NOT NULL,
        sensor_type_id INTEGER NOT NULL,
        value REAL,
        agg TEXT NOT NULL,
        window_sec INTEGER NOT NULL,
        UNIQUE(ts_ms, node_id, sensor_type_id, agg),
        FOREIGN KEY (node_id) REFERENCES node_name(id) ON DELETE CASCADE,
        FOREIGN KEY (sensor_type_id) REFERENCES sensor_type(id) ON DELETE RESTRICT
      );

      CREATE INDEX IF NOT EXISTS idx_node_values_ts ON node_values(ts_ms);
      CREATE INDEX IF NOT EXISTS idx_ghavg_ts ON greenhouse_average(ts_ms);
    "#)?;

    Ok(conn)
}

fn ensure_greenhouse(conn: &Connection, gh_id: u16) -> rusqlite::Result<()> {
    conn.execute("INSERT OR IGNORE INTO greenhouse_id(id) VALUES (?1)", params![gh_id])?;
    Ok(())
}
fn ensure_node(conn: &Connection, gh_id: u16, node_id: u16, label: &str) -> rusqlite::Result<i64> {
    ensure_greenhouse(conn, gh_id)?;
    conn.execute(
        "INSERT OR IGNORE INTO node_name(greenhouse_id,node_id,label) VALUES (?1,?2,?3)",
        params![gh_id, node_id, label],
    )?;
    conn.query_row(
        "SELECT id FROM node_name WHERE greenhouse_id=?1 AND node_id=?2",
        params![gh_id, node_id],
        |r| r.get::<_, i64>(0),
    )
}
fn ensure_sensor(conn: &Connection, key: &str, unit: &str) -> rusqlite::Result<i64> {
    conn.execute(
        "INSERT OR IGNORE INTO sensor_type(key,unit) VALUES (?1,?2)",
        params![key, unit],
    )?;
    conn.query_row(
        "SELECT id FROM sensor_type WHERE key=?1",
        params![key],
        |r| r.get::<_, i64>(0),
    )
}

fn label_for(node_id: u16) -> &'static str {
    match node_id {
        65001 => "Outdoor_Node",
        1 => "node01", 2 => "node02", 3 => "node03", 4 => "node04",
        5 => "node05", 6 => "node06", 7 => "node07", 8 => "node08",
        9 => "node09", 10 => "node10", 11 => "node11", 12 => "node12",
        _ => "nodeXX",
    }
}

fn insert_node_field(conn: &Connection, ts: i64, node_rowid: i64,
                     key: &str, unit: &str, val: Option<f32>) {
    if let Ok(st_id) = ensure_sensor(conn, key, unit) {
        if let Err(e) = conn.execute(
            "INSERT OR IGNORE INTO node_values(ts_ms,node_id,sensor_type_id,value,agg,window_sec)
             VALUES (?1,?2,?3,?4,'rolling_60s',60)",
            params![ts, node_rowid, st_id, r2(val)],
        ) {
            eprintln!("[DB] skip node field {key}: {e}");
        }
    } else {
        eprintln!("[DB] skip sensor ensure for key={key}");
    }
}

fn insert_gh_field(conn: &Connection, ts: i64, gh_id: u16,
                   key: &str, unit: &str, val: Option<f32>, nodes: usize) {
    if ensure_greenhouse(conn, gh_id).is_err() {
        eprintln!("[DB] skip greenhouse ensure gh_id={gh_id}");
        return;
    }
    if let Ok(st_id) = ensure_sensor(conn, key, unit) {
        if let Err(e) = conn.execute(
            "INSERT OR IGNORE INTO greenhouse_average
             (ts_ms,greenhouse_id,sensor_type_id,value,nodes,agg,window_sec)
             VALUES (?1,?2,?3,?4,?5,'rolling_60s',60)",
            params![ts, gh_id, st_id, r2(val), nodes as i64],
        ) {
            eprintln!("[DB] skip gh field {key}: {e}");
        }
    } else {
        eprintln!("[DB] skip gh sensor ensure for key={key}");
    }
}

/// Blocking batch flush inside a transaction (spawn_blocking caller).
/// Bad rows are logged and skipped; commit still happens.
fn flush_batch(db_path: &str, batch_nodes: Vec<NodeAvg>, batch_gh: Vec<GhAvg>) {
    if batch_nodes.is_empty() && batch_gh.is_empty() { return; }
    let abs = absolute_path(db_path);
    let Ok(conn) = open_and_init(abs.to_str().unwrap_or(db_path)) else {
        eprintln!("[DB] open/init failed at {}", abs.display());
        return;
    };
    let Ok(tx) = conn.unchecked_transaction() else {
        eprintln!("[DB] begin tx failed at {}", abs.display());
        return;
    };
    let ts = now_ms();

    for na in batch_nodes {
        match ensure_node(&tx, na.greenhouse_id, na.node_id, label_for(na.node_id)) {
            Ok(node_rowid) => {
                insert_node_field(&tx, ts, node_rowid, "air_temp_c", "C",   na.air_temp_c);
                insert_node_field(&tx, ts, node_rowid, "leaf_temp_c","C",   na.leaf_temp_c);
                insert_node_field(&tx, ts, node_rowid, "bag_temp_c", "C",   na.bag_temp_c);
                insert_node_field(&tx, ts, node_rowid, "air_rh_pct", "%",   na.air_rh_pct);
                insert_node_field(&tx, ts, node_rowid, "bag_rh1_pct","%",   na.bag_rh1_pct);
                insert_node_field(&tx, ts, node_rowid, "bag_rh2_pct","%",   na.bag_rh2_pct);
                insert_node_field(&tx, ts, node_rowid, "bag_rh3_pct","%",   na.bag_rh3_pct);
                insert_node_field(&tx, ts, node_rowid, "bag_rh4_pct","%",   na.bag_rh4_pct);
                insert_node_field(&tx, ts, node_rowid, "bag_rh_avg_pct","%",na.bag_rh_avg_pct);
                insert_node_field(&tx, ts, node_rowid, "par_value",  "",    na.par_value);
                insert_node_field(&tx, ts, node_rowid, "weight_g",   "",    na.weight_g);
                insert_node_field(&tx, ts, node_rowid, "ea_air_kpa", "kPa", na.ea_air_kpa);
                insert_node_field(&tx, ts, node_rowid, "ea_leaf_kpa","kPa", na.ea_leaf_kpa);
                insert_node_field(&tx, ts, node_rowid, "es_kpa",     "kPa", na.es_kpa);
                insert_node_field(&tx, ts, node_rowid, "vpd_kpa",    "kPa", na.vpd_kpa);
            }
            Err(e) => eprintln!("[DB] skip node ensure gh={} node={}: {e}", na.greenhouse_id, na.node_id),
        }
    }

    for ga in batch_gh {
        insert_gh_field(&tx, ts, ga.greenhouse_id, "air_temp_c","C",   ga.air_temp_c,   ga.nodes);
        insert_gh_field(&tx, ts, ga.greenhouse_id, "leaf_temp_c","C",  ga.leaf_temp_c,  ga.nodes);
        insert_gh_field(&tx, ts, ga.greenhouse_id, "bag_temp_c","C",   ga.bag_temp_c,   ga.nodes);
        insert_gh_field(&tx, ts, ga.greenhouse_id, "air_rh_pct","%",   ga.air_rh_pct,   ga.nodes);
        insert_gh_field(&tx, ts, ga.greenhouse_id, "bag_rh1_pct","%",  ga.bag_rh1_pct,  ga.nodes);
        insert_gh_field(&tx, ts, ga.greenhouse_id, "bag_rh2_pct","%",  ga.bag_rh2_pct,  ga.nodes);
        insert_gh_field(&tx, ts, ga.greenhouse_id, "bag_rh3_pct","%",  ga.bag_rh3_pct,  ga.nodes);
        insert_gh_field(&tx, ts, ga.greenhouse_id, "bag_rh4_pct","%",  ga.bag_rh4_pct,  ga.nodes);
        insert_gh_field(&tx, ts, ga.greenhouse_id, "bag_rh_avg_pct","%",ga.bag_rh_avg_pct,ga.nodes);
        insert_gh_field(&tx, ts, ga.greenhouse_id, "par_value","",      ga.par_value,    ga.nodes);
        insert_gh_field(&tx, ts, ga.greenhouse_id, "weight_g","",       ga.weight_g,     ga.nodes);
        insert_gh_field(&tx, ts, ga.greenhouse_id, "ea_air_kpa","kPa",  ga.ea_air_kpa,   ga.nodes);
        insert_gh_field(&tx, ts, ga.greenhouse_id, "ea_leaf_kpa","kPa", ga.ea_leaf_kpa,  ga.nodes);
        insert_gh_field(&tx, ts, ga.greenhouse_id, "es_kpa","kPa",      ga.es_kpa,       ga.nodes);
        insert_gh_field(&tx, ts, ga.greenhouse_id, "vpd_kpa","kPa",     ga.vpd_kpa,      ga.nodes);
    }

    if let Err(e) = tx.commit() {
        eprintln!("[DB] commit failed (batch skipped): {e}");
    }
}

/// Public async task:
/// - `rx_nodeavg`: NodeAvg stream (per-node 60s) from aggregator
/// - `rx_ghavg`: GhAvg stream (per-greenhouse 60s) from greenhouse aggregator
/// - Batches and flushes every 1s or 512 msgs via spawn_blocking (keeps hot path non-blocking)
pub async fn run_storage(
    db_path: &'static str,
    mut rx_nodeavg: mpsc::Receiver<NodeAvg>,
    mut rx_ghavg: mpsc::Receiver<GhAvg>,
) {
    let abs = absolute_path(db_path);
    println!("[DB] Using database at: {}", abs.display());

    // Ensure DB exists (blocking once)
    match tokio::task::spawn_blocking({
        let path = abs.clone();
        move || open_and_init(path.to_str().unwrap_or(db_path))
    }).await {
        Ok(Ok(_)) => (),
        Ok(Err(e)) => {
            eprintln!("[DB] init error at {}: {}", abs.display(), e);
            return;
        }
        Err(e) => {
            eprintln!("[DB] init join error: {}", e);
            return;
        }
    }

    const BATCH_SIZE: usize = 512;
    const FLUSH_EVERY: Duration = Duration::from_secs(1);

    let mut batch_nodes: Vec<NodeAvg> = Vec::with_capacity(256);
    let mut batch_gh: Vec<GhAvg> = Vec::with_capacity(128);
    let mut tick = interval(FLUSH_EVERY);

    loop {
        tokio::select! {
            Some(na) = rx_nodeavg.recv() => {
                batch_nodes.push(na);
                if batch_nodes.len() + batch_gh.len() >= BATCH_SIZE {
                    let bn = std::mem::take(&mut batch_nodes);
                    let bg = std::mem::take(&mut batch_gh);
                    let path = abs.clone();
                    let _ = tokio::task::spawn_blocking(move || flush_batch(path.to_str().unwrap(), bn, bg)).await;
                }
            }
            Some(ga) = rx_ghavg.recv() => {
                batch_gh.push(ga);
                if batch_nodes.len() + batch_gh.len() >= BATCH_SIZE {
                    let bn = std::mem::take(&mut batch_nodes);
                    let bg = std::mem::take(&mut batch_gh);
                    let path = abs.clone();
                    let _ = tokio::task::spawn_blocking(move || flush_batch(path.to_str().unwrap(), bn, bg)).await;
                }
            }
            _ = tick.tick() => {
                if !(batch_nodes.is_empty() && batch_gh.is_empty()) {
                    let bn = std::mem::take(&mut batch_nodes);
                    let bg = std::mem::take(&mut batch_gh);
                    let path = abs.clone();
                    let _ = tokio::task::spawn_blocking(move || flush_batch(path.to_str().unwrap(), bn, bg)).await;
                }
            }
            else => break,
        }
    }
}
