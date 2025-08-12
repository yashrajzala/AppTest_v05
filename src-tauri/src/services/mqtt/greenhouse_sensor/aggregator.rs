//! 60-second rolling average aggregator.
//! - Non-blocking: receives decoded samples via mpsc (producer never awaits).
//! - True rolling window by time (Instant), robust to jitter and gaps.
//! - Prints all field averages per node once every 60s.
//! - Handles offline/missing data gracefully (prints n/a when no samples).

use std::{collections::{HashMap, VecDeque}, time::Duration};
use tokio::sync::mpsc;
use tokio::time::{Instant, interval};

use super::decoder::Decoded;

const WINDOW: Duration = Duration::from_secs(60);
const MAX_SAMPLES_PER_NODE: usize = 64; // ample for 10s cadence

#[derive(Debug)]
struct TimedSample {
    at: Instant,
    data: Decoded,
}

#[derive(Debug)]
enum NodeKind {
    Standard,
    Outdoor,
}

#[derive(Debug)]
struct NodeWindow {
    kind: NodeKind,
    label: &'static str,
    buf: VecDeque<TimedSample>,
}

impl NodeWindow {
    fn new(kind: NodeKind, label: &'static str) -> Self {
        Self { kind, label, buf: VecDeque::with_capacity(8) }
    }

    /// Push a sample and drop anything older than WINDOW.
    fn push_and_prune(&mut self, now: Instant, data: Decoded) {
        self.buf.push_back(TimedSample { at: now, data });
        while let Some(front) = self.buf.front() {
            if now.duration_since(front.at) > WINDOW {
                self.buf.pop_front();
            } else {
                break;
            }
        }
        // Avoid unbounded growth on unusual cadences.
        while self.buf.len() > MAX_SAMPLES_PER_NODE {
            self.buf.pop_front();
        }
    }
}

/// Helper: mean(s) where `s` are iterated floats; skips NaN/inf.
#[inline]
fn mean_sum_count(sum: f64, cnt: u32) -> Option<f64> {
    if cnt == 0 { None } else { Some(sum / (cnt as f64)) }
}

#[inline]
fn acc(opt: f32, sum: &mut f64, cnt: &mut u32) {
    let v = opt as f64;
    if v.is_finite() {
        *sum += v;
        *cnt += 1;
    }
}

pub async fn run_rolling_avg(mut rx: mpsc::Receiver<Decoded>) {
    // Key by (greenhouse_id, node_id) to be future-proof.
    let mut nodes: HashMap<(u16, u16), NodeWindow> = HashMap::new();

    // Print exactly every 60 seconds.
    let mut tick = interval(WINDOW);
    // Align first tick to 60s from now (skip immediate print).
    tick.tick().await;

    loop {
        tokio::select! {
            // Receive decoded sample
            maybe_msg = rx.recv() => {
                if let Some(msg) = maybe_msg {
                    let now = Instant::now();

                    // Identify node key and kind
                    let (key, kind, label) = match msg {
                        Decoded::Standard { greenhouse_id, node_id, label, .. } =>
                            ((greenhouse_id, node_id), NodeKind::Standard, label),
                        Decoded::Outdoor  { greenhouse_id, node_id, label, .. } =>
                            ((greenhouse_id, node_id), NodeKind::Outdoor,  label),
                    };

                    // Upsert window and push
                    let entry = nodes.entry(key).or_insert_with(|| NodeWindow::new(kind, label));
                    // If kind changes (shouldn't), keep first seen kind for stability.
                    entry.push_and_prune(now, msg);
                } else {
                    // Channel closed; keep printing with whatever we have.
                }
            }

            // Once per 60 seconds, compute and print the rolling averages.
            _ = tick.tick() => {
                for ((_gh, _node), window) in nodes.iter_mut() {
                    // Ensure window contains only last WINDOW
                    let now = Instant::now();
                    while let Some(front) = window.buf.front() {
                        if now.duration_since(front.at) > WINDOW {
                            window.buf.pop_front();
                        } else { break; }
                    }
                    let samples = window.buf.len();

                    match window.kind {
                        NodeKind::Standard => {
                            // Accumulators (sum, count)
                            let (mut air_t_s, mut air_t_c)       = (0.0, 0);
                            let (mut leaf_t_s, mut leaf_t_c)     = (0.0, 0);
                            let (mut bag_t_s, mut bag_t_c)       = (0.0, 0);
                            let (mut air_rh_s, mut air_rh_c)     = (0.0, 0);
                            let (mut brh1_s, mut brh1_c)         = (0.0, 0);
                            let (mut brh2_s, mut brh2_c)         = (0.0, 0);
                            let (mut brh3_s, mut brh3_c)         = (0.0, 0);
                            let (mut brh4_s, mut brh4_c)         = (0.0, 0);
                            let (mut brh_avg_s, mut brh_avg_c)   = (0.0, 0);
                            let (mut par_s, mut par_c)           = (0.0, 0);
                            let (mut weight_s, mut weight_c)     = (0.0, 0);
                            let (mut ea_air_s, mut ea_air_c)     = (0.0, 0);
                            let (mut ea_leaf_s, mut ea_leaf_c)   = (0.0, 0);
                            let (mut es_s, mut es_c)             = (0.0, 0);
                            let (mut vpd_s, mut vpd_c)           = (0.0, 0);

                            for s in window.buf.iter() {
                                if let Decoded::Standard {
                                    air_temp_c, leaf_temp_c, bag_temp_c, air_rh_pct,
                                    bag_rh1_pct, bag_rh2_pct, bag_rh3_pct, bag_rh4_pct, bag_rh_avg_pct,
                                    par_value, weight_g, ea_air_kpa, ea_leaf_kpa, es_kpa, vpd_kpa, ..
                                } = s.data {
                                    acc(air_temp_c, &mut air_t_s, &mut air_t_c);
                                    acc(leaf_temp_c, &mut leaf_t_s, &mut leaf_t_c);
                                    acc(bag_temp_c,  &mut bag_t_s,  &mut bag_t_c);
                                    acc(air_rh_pct,  &mut air_rh_s, &mut air_rh_c);
                                    acc(bag_rh1_pct, &mut brh1_s,   &mut brh1_c);
                                    acc(bag_rh2_pct, &mut brh2_s,   &mut brh2_c);
                                    acc(bag_rh3_pct, &mut brh3_s,   &mut brh3_c);
                                    acc(bag_rh4_pct, &mut brh4_s,   &mut brh4_c);
                                    acc(bag_rh_avg_pct, &mut brh_avg_s, &mut brh_avg_c);
                                    acc(par_value as f32, &mut par_s, &mut par_c);
                                    acc(weight_g as f32,  &mut weight_s, &mut weight_c);
                                    acc(ea_air_kpa,  &mut ea_air_s,  &mut ea_air_c);
                                    acc(ea_leaf_kpa, &mut ea_leaf_s, &mut ea_leaf_c);
                                    acc(es_kpa,      &mut es_s,      &mut es_c);
                                    acc(vpd_kpa,     &mut vpd_s,     &mut vpd_c);
                                }
                            }

                            // Print: all fields, even if n/a.
                            println!(
                                "[AVG-60s] {} | Samples: {} | Air_temp: {} | Leaf_temp: {} | Bag_temp: {} | Air_Rh: {} | Bag_Rh1: {} | Bag_Rh2: {} | Bag_Rh3: {} | Bag_Rh4: {} | Bag_Rh_Avg: {} | PAR: {} | Weight: {} | Ea_air: {} | Ea_leaf: {} | Es: {} | VPD: {}",
                                window.label, samples,
                                fmt(mean_sum_count(air_t_s,   air_t_c),   "C",   1),
                                fmt(mean_sum_count(leaf_t_s,  leaf_t_c),  "C",   1),
                                fmt(mean_sum_count(bag_t_s,   bag_t_c),   "C",   1),
                                fmt(mean_sum_count(air_rh_s,  air_rh_c),  "%",   1),
                                fmt(mean_sum_count(brh1_s,    brh1_c),    "%",   1),
                                fmt(mean_sum_count(brh2_s,    brh2_c),    "%",   1),
                                fmt(mean_sum_count(brh3_s,    brh3_c),    "%",   1),
                                fmt(mean_sum_count(brh4_s,    brh4_c),    "%",   1),
                                fmt(mean_sum_count(brh_avg_s, brh_avg_c), "%",   1),
                                fmt(mean_sum_count(par_s,     par_c),     "",    1),
                                fmt(mean_sum_count(weight_s,  weight_c),  "",    1),
                                fmt(mean_sum_count(ea_air_s,  ea_air_c),  "kPa", 3),
                                fmt(mean_sum_count(ea_leaf_s, ea_leaf_c), "kPa", 3),
                                fmt(mean_sum_count(es_s,      es_c),      "kPa", 3),
                                fmt(mean_sum_count(vpd_s,     vpd_c),     "kPa", 3),
                            );
                        }
                        NodeKind::Outdoor => {
                            let (mut air_t_s, mut air_t_c)   = (0.0, 0);
                            let (mut air_rh_s, mut air_rh_c) = (0.0, 0);
                            let (mut par_s, mut par_c)       = (0.0, 0);
                            let (mut ea_air_s, mut ea_air_c) = (0.0, 0);
                            let (mut es_s, mut es_c)         = (0.0, 0);

                            for s in window.buf.iter() {
                                if let Decoded::Outdoor {
                                    air_temp_c, air_rh_pct, par_value, ea_air_kpa, es_kpa, ..
                                } = s.data {
                                    acc(air_temp_c, &mut air_t_s, &mut air_t_c);
                                    acc(air_rh_pct, &mut air_rh_s, &mut air_rh_c);
                                    acc(par_value as f32, &mut par_s, &mut par_c);
                                    acc(ea_air_kpa, &mut ea_air_s, &mut ea_air_c);
                                    acc(es_kpa,     &mut es_s,     &mut es_c);
                                }
                            }

                            println!(
                                "[AVG-60s] {} | Samples: {} | Air_temp: {} | Air_Rh: {} | PAR: {} | Ea_air: {} | Es: {}",
                                window.label, samples,
                                fmt(mean_sum_count(air_t_s, air_t_c), "C",   1),
                                fmt(mean_sum_count(air_rh_s, air_rh_c), "%", 1),
                                fmt(mean_sum_count(par_s, par_c), "",        1),
                                fmt(mean_sum_count(ea_air_s, ea_air_c), "kPa", 3),
                                fmt(mean_sum_count(es_s, es_c),       "kPa",  3),
                            );
                        }
                    }
                }
            }
        }
    }
}

/// Compact formatter: "n/a" if None else with unit & precision.
fn fmt(v: Option<f64>, unit: &str, dp: usize) -> String {
    match v {
        Some(x) if x.is_finite() => match dp {
            0 => if unit.is_empty() { format!("{:.0}", x) } else { format!("{:.0} {}", x, unit) },
            1 => if unit.is_empty() { format!("{:.1}", x) } else { format!("{:.1} {}", x, unit) },
            2 => if unit.is_empty() { format!("{:.2}", x) } else { format!("{:.2} {}", x, unit) },
            _ => if unit.is_empty() { format!("{:.3}", x) } else { format!("{:.3} {}", x, unit) },
        },
        _ => "n/a".to_string(),
    }
}
