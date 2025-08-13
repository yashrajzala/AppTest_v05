//! Per-node 60s rolling averages (time-based, resilient to dropouts).
//! - Non-blocking: decoded samples come in via mpsc::Receiver<Decoded>.
//! - Every 60s we compute means for the last 60s window and:
//!     * Print one compact line per node with **two decimals** everywhere.
//!     * Emit NodeAvg to BOTH: DB writer and greenhouse aggregator.
//! - RAM-only buffers, bounded, no panics.

use std::{collections::{HashMap, VecDeque}, time::Duration};
use tokio::sync::mpsc;
use tokio::time::{Instant, interval};

use super::decoder::Decoded;

// 60-second window
const WINDOW: Duration = Duration::from_secs(60);
const MAX_SAMPLES_PER_NODE: usize = 64; // ~6 samples/min, headroom

#[derive(Debug)]
struct TimedSample {
    at: Instant,
    data: Decoded,
}

#[derive(Debug, Clone, Copy)]
enum NodeKind {
    Standard,
    Outdoor,
}

#[derive(Debug)]
struct NodeWindow {
    kind: NodeKind,
    ids: (u16, u16), // (greenhouse_id, node_id)
    buf: VecDeque<TimedSample>,
}

impl NodeWindow {
    fn new(kind: NodeKind, ids: (u16,u16)) -> Self {
        Self { kind, ids, buf: VecDeque::with_capacity(8) }
    }
    fn push_and_prune(&mut self, now: Instant, data: Decoded) {
        self.buf.push_back(TimedSample { at: now, data });
        while let Some(front) = self.buf.front() {
            if now.duration_since(front.at) > WINDOW { self.buf.pop_front(); } else { break; }
        }
        while self.buf.len() > MAX_SAMPLES_PER_NODE { self.buf.pop_front(); }
    }
}

/// Per-node 60s snapshot (all fields optional to reflect missing data).
#[derive(Debug, Clone, Copy)]
pub struct NodeAvg {
    pub greenhouse_id: u16,
    pub node_id: u16,
    pub at: Instant,

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
}

#[inline] fn mean(sum: f64, cnt: u32) -> Option<f32> {
    if cnt == 0 { None } else { Some((sum / (cnt as f64)) as f32) }
}
#[inline] fn acc(v: f32, sum: &mut f64, cnt: &mut u32) {
    let x = v as f64; if x.is_finite() { *sum += x; *cnt += 1; }
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

/// Public task:
/// - rx_decoded: incoming Decoded samples from subscriber
/// - tx_nodeavg_db: NodeAvg stream to DB writer
/// - tx_nodeavg_gh: NodeAvg stream to greenhouse aggregator
pub async fn run_rolling_avg(
    mut rx_decoded: mpsc::Receiver<Decoded>,
    tx_nodeavg_db: mpsc::Sender<NodeAvg>,
    tx_nodeavg_gh: mpsc::Sender<NodeAvg>,
) {
    let mut nodes: HashMap<(u16, u16), NodeWindow> = HashMap::new();
    let mut tick = interval(WINDOW);
    tick.tick().await; // align first output to +60s

    loop {
        tokio::select! {
            maybe_msg = rx_decoded.recv() => {
                if let Some(msg) = maybe_msg {
                    let now = Instant::now();
                    let (key, kind) = match msg {
                        Decoded::Standard { greenhouse_id, node_id, .. } =>
                            ((greenhouse_id, node_id), NodeKind::Standard),
                        Decoded::Outdoor  { greenhouse_id, node_id, .. } =>
                            ((greenhouse_id, node_id), NodeKind::Outdoor),
                    };
                    nodes.entry(key).or_insert_with(|| NodeWindow::new(kind, key))
                         .push_and_prune(now, msg);
                }
            }
            _ = tick.tick() => {
                let now = Instant::now();
                for (_key, win) in nodes.iter_mut() {
                    while let Some(front) = win.buf.front() {
                        if now.duration_since(front.at) > WINDOW { win.buf.pop_front(); } else { break; }
                    }
                    let samples = win.buf.len();
                    if samples == 0 { continue; }

                    match win.kind {
                        NodeKind::Standard => {
                            // sums + counts
                            let (mut air_t_s,mut air_t_c)=(0.0,0); let (mut leaf_t_s,mut leaf_t_c)=(0.0,0);
                            let (mut bag_t_s,mut bag_t_c)=(0.0,0); let (mut air_rh_s,mut air_rh_c)=(0.0,0);
                            let (mut brh1_s,mut brh1_c)=(0.0,0); let (mut brh2_s,mut brh2_c)=(0.0,0);
                            let (mut brh3_s,mut brh3_c)=(0.0,0); let (mut brh4_s,mut brh4_c)=(0.0,0);
                            let (mut brh_avg_s,mut brh_avg_c)=(0.0,0); let (mut par_s,mut par_c)=(0.0,0);
                            let (mut weight_s,mut weight_c)=(0.0,0); let (mut ea_air_s,mut ea_air_c)=(0.0,0);
                            let (mut ea_leaf_s,mut ea_leaf_c)=(0.0,0); let (mut es_s,mut es_c)=(0.0,0);
                            let (mut vpd_s,mut vpd_c)=(0.0,0);

                            for s in win.buf.iter() {
                                if let Decoded::Standard {
                                    air_temp_c, leaf_temp_c, bag_temp_c, air_rh_pct,
                                    bag_rh1_pct, bag_rh2_pct, bag_rh3_pct, bag_rh4_pct, bag_rh_avg_pct,
                                    par_value, weight_g, ea_air_kpa, ea_leaf_kpa, es_kpa, vpd_kpa, ..
                                } = s.data {
                                    acc(air_temp_c, &mut air_t_s, &mut air_t_c);
                                    acc(leaf_temp_c,&mut leaf_t_s,&mut leaf_t_c);
                                    acc(bag_temp_c, &mut bag_t_s, &mut bag_t_c);
                                    acc(air_rh_pct, &mut air_rh_s, &mut air_rh_c);
                                    acc(bag_rh1_pct,&mut brh1_s,  &mut brh1_c);
                                    acc(bag_rh2_pct,&mut brh2_s,  &mut brh2_c);
                                    acc(bag_rh3_pct,&mut brh3_s,  &mut brh3_c);
                                    acc(bag_rh4_pct,&mut brh4_s,  &mut brh4_c);
                                    acc(bag_rh_avg_pct,&mut brh_avg_s,&mut brh_avg_c);
                                    acc(par_value as f32,&mut par_s,&mut par_c);
                                    acc(weight_g as f32,&mut weight_s,&mut weight_c);
                                    acc(ea_air_kpa,  &mut ea_air_s,  &mut ea_air_c);
                                    acc(ea_leaf_kpa, &mut ea_leaf_s, &mut ea_leaf_c);
                                    acc(es_kpa,      &mut es_s,      &mut es_c);
                                    acc(vpd_kpa,     &mut vpd_s,     &mut vpd_c);
                                }
                            }

                            let na = NodeAvg {
                                greenhouse_id: win.ids.0, node_id: win.ids.1, at: now,
                                air_temp_c: mean(air_t_s, air_t_c),   leaf_temp_c: mean(leaf_t_s, leaf_t_c),
                                bag_temp_c: mean(bag_t_s, bag_t_c),   air_rh_pct:  mean(air_rh_s, air_rh_c),
                                bag_rh1_pct: mean(brh1_s, brh1_c),    bag_rh2_pct: mean(brh2_s, brh2_c),
                                bag_rh3_pct: mean(brh3_s, brh3_c),    bag_rh4_pct: mean(brh4_s, brh4_c),
                                bag_rh_avg_pct: mean(brh_avg_s, brh_avg_c),
                                par_value: mean(par_s, par_c),        weight_g:  mean(weight_s, weight_c),
                                ea_air_kpa: mean(ea_air_s, ea_air_c), ea_leaf_kpa: mean(ea_leaf_s, ea_leaf_c),
                                es_kpa: mean(es_s, es_c),             vpd_kpa: mean(vpd_s, vpd_c),
                            };

                            println!(
                              "[AVG-60s] GH:{} Node:{} | Samples:{} | Air:{} | Leaf:{} | Bag:{} | RH:{} | BRH1:{} | BRH2:{} | BRH3:{} | BRH4:{} | BRH_avg:{} | PAR:{} | W:{} | Ea_air:{} | Ea_leaf:{} | Es:{} | VPD:{}",
                              win.ids.0, win.ids.1, samples,
                              fmt_opt2(na.air_temp_c, "C"),
                              fmt_opt2(na.leaf_temp_c, "C"),
                              fmt_opt2(na.bag_temp_c, "C"),
                              fmt_opt2(na.air_rh_pct, "%"),
                              fmt_opt2(na.bag_rh1_pct, "%"),
                              fmt_opt2(na.bag_rh2_pct, "%"),
                              fmt_opt2(na.bag_rh3_pct, "%"),
                              fmt_opt2(na.bag_rh4_pct, "%"),
                              fmt_opt2(na.bag_rh_avg_pct, "%"),
                              fmt_opt2(na.par_value, ""),
                              fmt_opt2(na.weight_g, ""),
                              fmt_opt2(na.ea_air_kpa, "kPa"),
                              fmt_opt2(na.ea_leaf_kpa, "kPa"),
                              fmt_opt2(na.es_kpa, "kPa"),
                              fmt_opt2(na.vpd_kpa, "kPa"),
                            );

                            let _ = tx_nodeavg_db.try_send(na);
                            let _ = tx_nodeavg_gh.try_send(na);
                        }
                        NodeKind::Outdoor => {
                            let (mut air_t_s,mut air_t_c)=(0.0,0); let (mut air_rh_s,mut air_rh_c)=(0.0,0);
                            let (mut par_s,mut par_c)=(0.0,0); let (mut ea_air_s,mut ea_air_c)=(0.0,0);
                            let (mut es_s,mut es_c)=(0.0,0);

                            for s in win.buf.iter() {
                                if let Decoded::Outdoor { air_temp_c, air_rh_pct, par_value, ea_air_kpa, es_kpa, .. } = s.data {
                                    acc(air_temp_c, &mut air_t_s, &mut air_t_c);
                                    acc(air_rh_pct, &mut air_rh_s, &mut air_rh_c);
                                    acc(par_value as f32, &mut par_s, &mut par_c);
                                    acc(ea_air_kpa, &mut ea_air_s, &mut ea_air_c);
                                    acc(es_kpa, &mut es_s, &mut es_c);
                                }
                            }

                            let na = NodeAvg {
                                greenhouse_id: win.ids.0, node_id: win.ids.1, at: now,
                                air_temp_c: mean(air_t_s, air_t_c),  leaf_temp_c: None,
                                bag_temp_c: None,                    air_rh_pct: mean(air_rh_s, air_rh_c),
                                bag_rh1_pct: None, bag_rh2_pct: None, bag_rh3_pct: None, bag_rh4_pct: None,
                                bag_rh_avg_pct: None,
                                par_value: mean(par_s, par_c),       weight_g: None,
                                ea_air_kpa: mean(ea_air_s, ea_air_c), ea_leaf_kpa: None,
                                es_kpa: mean(es_s, es_c),            vpd_kpa: None,
                            };

                            println!(
                                "[AVG-60s] GH:{} Node:{} | Samples:{} | Air:{} | RH:{} | PAR:{} | Ea_air:{} | Es:{}",
                                win.ids.0, win.ids.1, samples,
                                fmt_opt2(na.air_temp_c, "C"),
                                fmt_opt2(na.air_rh_pct, "%"),
                                fmt_opt2(na.par_value, ""),
                                fmt_opt2(na.ea_air_kpa, "kPa"),
                                fmt_opt2(na.es_kpa, "kPa"),
                            );

                            let _ = tx_nodeavg_db.try_send(na);
                            let _ = tx_nodeavg_gh.try_send(na);
                        }
                    }
                }
            }
        }
    }
}
