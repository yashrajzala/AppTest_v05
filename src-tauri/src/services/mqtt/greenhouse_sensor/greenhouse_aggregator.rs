//! Greenhouse-level 60s averages.
//! - Consumes NodeAvg (per-node snapshots).
//! - Every 60s, averages available fields across freshest nodes.
//! - Prints with two decimals; emits GhAvg to DB.

use std::{collections::HashMap, time::Duration};
use tokio::sync::mpsc;
use tokio::time::{Instant, interval};

use super::aggregator::NodeAvg;

const WINDOW: Duration = Duration::from_secs(60);
const STALE_GRACE: Duration = Duration::from_secs(5); // include node avgs if <= 65s old

#[derive(Debug, Clone, Copy)]
pub struct GhAvg {
    pub greenhouse_id: u16,
    pub air_temp_c: Option<f32>,
    pub leaf_temp_c: Option<f32>,
    pub bag_temp_c: Option<f32>,
    pub air_rh_pct: Option<f32>,
    pub bag_rh1_pct: Option<f32>,
    pub bag_rh2_pct: Option<f32>,
    pub bag_rh3_pct: Option<f32>,
    pub bag_rh4_pct: Option<f32>,
    pub bag_rh_avg_pct: Option<f32>,
    pub par_value: Option<f32>,
    pub weight_g: Option<f32>,
    pub ea_air_kpa: Option<f32>,
    pub ea_leaf_kpa: Option<f32>,
    pub es_kpa: Option<f32>,
    pub vpd_kpa: Option<f32>,
    pub nodes: usize,
}

#[inline] fn mean(sum: f64, cnt: u32) -> Option<f32> {
    if cnt == 0 { None } else { Some((sum / (cnt as f64)) as f32) }
}
#[inline] fn acc_opt(v: Option<f32>, sum: &mut f64, cnt: &mut u32) {
    if let Some(x) = v { let y = x as f64; if y.is_finite() { *sum += y; *cnt += 1; } }
}
#[inline] fn round2(v: f32) -> f32 { (v * 100.0).round() / 100.0 }
fn fmt_opt2(v: Option<f32>, unit: &str) -> String {
    match v.map(round2) {
        Some(x) if x.is_finite() => {
            if unit.is_empty() { format!("Some({:.2})", x) }
            else { format!("Some({:.2}){}", x, unit) }
        }
        _ => "None".to_string(),
    }
}

struct GHState { nodes: HashMap<u16, NodeAvg> }
impl GHState { fn new() -> Self { Self { nodes: HashMap::new() } } }

pub async fn run_greenhouse_avg(
    mut rx_nodeavg: mpsc::Receiver<NodeAvg>,
    tx_ghavg_db: mpsc::Sender<GhAvg>,
) {
    let mut gh: HashMap<u16, GHState> = HashMap::new();
    let mut tick = interval(WINDOW);
    tick.tick().await;

    loop {
        tokio::select! {
            Some(na) = rx_nodeavg.recv() => {
                gh.entry(na.greenhouse_id).or_insert_with(GHState::new)
                  .nodes.insert(na.node_id, na);
            }
            _ = tick.tick() => {
                let now = Instant::now();
                for (gh_id, st) in gh.iter() {
                    let fresh: Vec<&NodeAvg> = st.nodes.values()
                        .filter(|v| now.duration_since(v.at) <= WINDOW + STALE_GRACE)
                        .collect();
                    let n_nodes = fresh.len();
                    if n_nodes == 0 {
                        println!("[GH-AVG-60s] GH:{} | No fresh node averages (last 60s)", gh_id);
                        continue;
                    }

                    macro_rules! acc_field {
                        ($getter:ident) => {{
                            let (mut s, mut c) = (0.0f64, 0u32);
                            for v in &fresh { acc_opt(v.$getter, &mut s, &mut c); }
                            mean(s, c)
                        }};
                    }

                    let air_temp_c     = acc_field!(air_temp_c);
                    let leaf_temp_c    = acc_field!(leaf_temp_c);
                    let bag_temp_c     = acc_field!(bag_temp_c);
                    let air_rh_pct     = acc_field!(air_rh_pct);
                    let bag_rh1_pct    = acc_field!(bag_rh1_pct);
                    let bag_rh2_pct    = acc_field!(bag_rh2_pct);
                    let bag_rh3_pct    = acc_field!(bag_rh3_pct);
                    let bag_rh4_pct    = acc_field!(bag_rh4_pct);
                    let bag_rh_avg_pct = acc_field!(bag_rh_avg_pct);
                    let par_value      = acc_field!(par_value);
                    let weight_g       = acc_field!(weight_g);
                    let ea_air_kpa     = acc_field!(ea_air_kpa);
                    let ea_leaf_kpa    = acc_field!(ea_leaf_kpa);
                    let es_kpa         = acc_field!(es_kpa);
                    let vpd_kpa        = acc_field!(vpd_kpa);

                    println!(
                        "[GH-AVG-60s] GH:{} | Nodes:{} | Air:{} | Leaf:{} | Bag:{} | RH:{} | BRH1:{} | BRH2:{} | BRH3:{} | BRH4:{} | BRH_avg:{} | PAR:{} | W:{} | Ea_air:{} | Ea_leaf:{} | Es:{} | VPD:{}",
                        gh_id, n_nodes,
                        fmt_opt2(air_temp_c, "C"),
                        fmt_opt2(leaf_temp_c, "C"),
                        fmt_opt2(bag_temp_c, "C"),
                        fmt_opt2(air_rh_pct, "%"),
                        fmt_opt2(bag_rh1_pct, "%"),
                        fmt_opt2(bag_rh2_pct, "%"),
                        fmt_opt2(bag_rh3_pct, "%"),
                        fmt_opt2(bag_rh4_pct, "%"),
                        fmt_opt2(bag_rh_avg_pct, "%"),
                        fmt_opt2(par_value, ""),
                        fmt_opt2(weight_g, ""),
                        fmt_opt2(ea_air_kpa, "kPa"),
                        fmt_opt2(ea_leaf_kpa, "kPa"),
                        fmt_opt2(es_kpa, "kPa"),
                        fmt_opt2(vpd_kpa, "kPa"),
                    );

                    let ga = GhAvg {
                        greenhouse_id: *gh_id,
                        air_temp_c, leaf_temp_c, bag_temp_c, air_rh_pct,
                        bag_rh1_pct, bag_rh2_pct, bag_rh3_pct, bag_rh4_pct, bag_rh_avg_pct,
                        par_value, weight_g, ea_air_kpa, ea_leaf_kpa, es_kpa, vpd_kpa,
                        nodes: n_nodes,
                    };
                    let _ = tx_ghavg_db.try_send(ga);
                }
            }
        }
    }
}
