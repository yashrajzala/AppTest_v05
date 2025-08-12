//! Ultra-fast, allocation-free binary decoders for greenhouse sensor payloads.
//! Matches Arduino `__attribute__((packed))` LE layouts exactly.

#[derive(Debug, Clone, Copy)]
pub enum Decoded {
    /// Standard in-greenhouse nodes (node01..nodeXX)
    Standard {
        greenhouse_id: u16,
        node_id: u16,
        label: &'static str, // "node01".."nodeXX"
        air_temp_c: f32,
        leaf_temp_c: f32,
        bag_temp_c: f32,
        air_rh_pct: f32,
        bag_rh1_pct: f32,
        bag_rh2_pct: f32,
        bag_rh3_pct: f32,
        bag_rh4_pct: f32,
        bag_rh_avg_pct: f32,
        par_value: u16,
        weight_g: u16,
        ea_air_kpa: f32,
        ea_leaf_kpa: f32,
        es_kpa: f32,
        vpd_kpa: f32,
    },
    /// Outdoor probe (node65001)
    Outdoor {
        greenhouse_id: u16,
        node_id: u16,         // always 65001
        label: &'static str,  // "Outdoor_Node"
        air_temp_c: f32,
        air_rh_pct: f32,
        par_value: u16,
        ea_air_kpa: f32,
        es_kpa: f32,
    },
}

/// Map numeric node_id -> stable label w/o allocations.
/// Extend table as needed.
#[inline]
fn node_label(node_id: u16) -> &'static str {
    match node_id {
        65001 => "Outdoor_Node",
        1 => "node01",  2 => "node02",  3 => "node03",  4 => "node04",
        5 => "node05",  6 => "node06",  7 => "node07",  8 => "node08",
        9 => "node09", 10 => "node10", 11 => "node11", 12 => "node12",
        _ => "nodeXX",
    }
}

#[inline]
fn u16_le_at(buf: &[u8], off: usize) -> Option<u16> {
    let s = buf.get(off..off.checked_add(2)?)?;
    Some(u16::from_le_bytes([s[0], s[1]]))
}

#[inline]
fn f32_le_at(buf: &[u8], off: usize) -> Option<f32> {
    let s = buf.get(off..off.checked_add(4)?)?;
    Some(f32::from_le_bytes([s[0], s[1], s[2], s[3]]))
}

/// Decode strictly per provided layouts. Returns None on any size/field error.
/// Never panics.
#[inline]
pub fn decode_payload(payload: &[u8]) -> Option<Decoded> {
    let greenhouse_id = u16_le_at(payload, 0)?;
    let node_id       = u16_le_at(payload, 2)?;

    if node_id == 65001 {
        // ---- Node65001Payload (22 bytes) ----
        if payload.len() < 22 { return None; }
        Some(Decoded::Outdoor {
            greenhouse_id,
            node_id,
            label: "Outdoor_Node",
            air_temp_c: f32_le_at(payload, 4)?,
            air_rh_pct: f32_le_at(payload, 8)?,
            par_value:  u16_le_at(payload, 12)?,
            ea_air_kpa: f32_le_at(payload, 14)?,
            es_kpa:     f32_le_at(payload, 18)?,
        })
    } else {
        // ---- SensorDataPayload (60 bytes) ----
        if payload.len() < 60 { return None; }
        Some(Decoded::Standard {
            greenhouse_id,
            node_id,
            label: node_label(node_id),
            air_temp_c:     f32_le_at(payload,  4)?,
            leaf_temp_c:    f32_le_at(payload,  8)?,
            bag_temp_c:     f32_le_at(payload, 12)?,
            air_rh_pct:     f32_le_at(payload, 16)?,
            bag_rh1_pct:    f32_le_at(payload, 20)?,
            bag_rh2_pct:    f32_le_at(payload, 24)?,
            bag_rh3_pct:    f32_le_at(payload, 28)?,
            bag_rh4_pct:    f32_le_at(payload, 32)?,
            bag_rh_avg_pct: f32_le_at(payload, 36)?,
            par_value:      u16_le_at(payload, 40)?,
            weight_g:       u16_le_at(payload, 42)?,
            ea_air_kpa:     f32_le_at(payload, 44)?,
            ea_leaf_kpa:    f32_le_at(payload, 48)?,
            es_kpa:         f32_le_at(payload, 52)?,
            vpd_kpa:        f32_le_at(payload, 56)?,
        })
    }
}
