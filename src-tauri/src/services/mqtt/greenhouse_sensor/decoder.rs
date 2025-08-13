//! Decode fixed little-endian binary payloads from ESP32 nodes.
//!
//! Layouts (packed):
//! - Standard nodes (01..04): 60 bytes
//!   u16 greenhouse_id, u16 node_id,
//!   f32 air_temp, f32 leaf_temp, f32 bag_temp, f32 air_rh,
//!   f32 bag_rh1, f32 bag_rh2, f32 bag_rh3, f32 bag_rh4, f32 bag_rh_avg,
//!   u16 par_value, u16 weight,
//!   f32 ea_air, f32 ea_leaf, f32 es, f32 vpd
//!
//! - Outdoor node (65001): 22 bytes
//!   u16 greenhouse_id, u16 node_id,
//!   f32 air_temp, f32 air_rh, u16 par_value, f32 ea_air, f32 es

#[derive(Debug, Clone, Copy)]
pub enum Decoded {
    Standard {
        greenhouse_id: u16,
        node_id: u16,
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
    Outdoor {
        greenhouse_id: u16,
        node_id: u16,
        air_temp_c: f32,
        air_rh_pct: f32,
        par_value: u16,
        ea_air_kpa: f32,
        es_kpa: f32,
    },
}

#[inline] fn rd_u16_le(b: &[u8], o: usize) -> Option<u16> {
    b.get(o..o+2).map(|s| u16::from_le_bytes([s[0], s[1]]))
}
#[inline] fn rd_f32_le(b: &[u8], o: usize) -> Option<f32> {
    let a = *b.get(o)?; let a1 = *b.get(o+1)?;
    let a2 = *b.get(o+2)?; let a3 = *b.get(o+3)?;
    Some(f32::from_le_bytes([a, a1, a2, a3]))
}

pub fn decode_payload(p: &[u8]) -> Option<Decoded> {
    match p.len() {
        60 => {
            // Standard
            let mut o = 0usize;
            let greenhouse_id = rd_u16_le(p, o)?; o += 2;
            let node_id       = rd_u16_le(p, o)?; o += 2;

            let air_temp_c    = rd_f32_le(p, o)?; o += 4;
            let leaf_temp_c   = rd_f32_le(p, o)?; o += 4;
            let bag_temp_c    = rd_f32_le(p, o)?; o += 4;
            let air_rh_pct    = rd_f32_le(p, o)?; o += 4;

            let bag_rh1_pct   = rd_f32_le(p, o)?; o += 4;
            let bag_rh2_pct   = rd_f32_le(p, o)?; o += 4;
            let bag_rh3_pct   = rd_f32_le(p, o)?; o += 4;
            let bag_rh4_pct   = rd_f32_le(p, o)?; o += 4;
            let bag_rh_avg_pct= rd_f32_le(p, o)?; o += 4;

            let par_value     = rd_u16_le(p, o)?; o += 2;
            let weight_g      = rd_u16_le(p, o)?; o += 2;

            let ea_air_kpa    = rd_f32_le(p, o)?; o += 4;
            let ea_leaf_kpa   = rd_f32_le(p, o)?; o += 4;
            let es_kpa        = rd_f32_le(p, o)?; o += 4;
            let vpd_kpa       = rd_f32_le(p, o)?; /*o += 4;*/

            Some(Decoded::Standard {
                greenhouse_id, node_id,
                air_temp_c, leaf_temp_c, bag_temp_c, air_rh_pct,
                bag_rh1_pct, bag_rh2_pct, bag_rh3_pct, bag_rh4_pct, bag_rh_avg_pct,
                par_value, weight_g, ea_air_kpa, ea_leaf_kpa, es_kpa, vpd_kpa
            })
        }
        22 => {
            // Outdoor
            let mut o = 0usize;
            let greenhouse_id = rd_u16_le(p, o)?; o += 2;
            let node_id       = rd_u16_le(p, o)?; o += 2;

            let air_temp_c    = rd_f32_le(p, o)?; o += 4;
            let air_rh_pct    = rd_f32_le(p, o)?; o += 4;
            let par_value     = rd_u16_le(p, o)?; o += 2;
            let ea_air_kpa    = rd_f32_le(p, o)?; o += 4;
            let es_kpa        = rd_f32_le(p, o)?; /*o += 4;*/

            Some(Decoded::Outdoor {
                greenhouse_id, node_id,
                air_temp_c, air_rh_pct, par_value, ea_air_kpa, es_kpa
            })
        }
        _ => None,
    }
}
