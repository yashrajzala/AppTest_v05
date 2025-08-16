# Greenhouse Sensor System - SeriesKey Reference

## Overview
This document lists all available SeriesKey values from the 60-second sensor aggregators and how to use them with each component.

## Available SeriesKey Values

### Temperature Sensors
| SeriesKey | Description | Unit | Range | Available In |
|-----------|-------------|------|-------|--------------|
| `air_temp_c` | Air temperature | °C | -40 to +80 | All nodes + Greenhouse |
| `leaf_temp_c` | Leaf temperature | °C | -40 to +80 | Standard nodes (01-04) + Greenhouse |
| `bag_temp_c` | Bag temperature | °C | -40 to +80 | Standard nodes (01-04) + Greenhouse |

### Humidity Sensors
| SeriesKey | Description | Unit | Range | Available In |
|-----------|-------------|------|-------|--------------|
| `air_rh_pct` | Air relative humidity | % | 0-100 | All nodes + Greenhouse |
| `bag_rh1_pct` | Bag RH sensor 1 | % | 0-100 | Standard nodes (01-04) + Greenhouse |
| `bag_rh2_pct` | Bag RH sensor 2 | % | 0-100 | Standard nodes (01-04) + Greenhouse |
| `bag_rh3_pct` | Bag RH sensor 3 | % | 0-100 | Standard nodes (01-04) + Greenhouse |
| `bag_rh4_pct` | Bag RH sensor 4 | % | 0-100 | Standard nodes (01-04) + Greenhouse |
| `bag_rh_avg_pct` | Average bag RH | % | 0-100 | Standard nodes (01-04) + Greenhouse |

### Environmental Sensors
| SeriesKey | Description | Unit | Range | Available In |
|-----------|-------------|------|-------|--------------|
| `par_value` | Photosynthetic Active Radiation | - | 0-65535 | All nodes + Greenhouse |
| `weight_g` | Weight | g | 0-65535 | Standard nodes (01-04) + Greenhouse |

### Pressure Sensors
| SeriesKey | Description | Unit | Range | Available In |
|-----------|-------------|------|-------|--------------|
| `ea_air_kpa` | Air vapor pressure | kPa | 0-10 | All nodes + Greenhouse |
| `ea_leaf_kpa` | Leaf vapor pressure | kPa | 0-10 | Standard nodes (01-04) + Greenhouse |
| `es_kpa` | Saturation vapor pressure | kPa | 0-10 | All nodes + Greenhouse |
| `vpd_kpa` | Vapor Pressure Deficit | kPa | 0-10 | Standard nodes (01-04) + Greenhouse |

## Node Types

### Standard Nodes (01-04)
- **Full sensor suite**: All temperature, humidity, environmental, and pressure sensors
- **Location**: Inside greenhouse
- **Update rate**: 60-second averages

### Outdoor Node (65001)
- **Limited sensors**: Air temperature, air humidity, PAR, air vapor pressure, saturation pressure
- **Location**: Outside greenhouse
- **Update rate**: 60-second averages

### Greenhouse Aggregator
- **Combined data**: Averages across all active standard nodes
- **Update rate**: 60-second averages
- **Use case**: Overall greenhouse conditions

## Component Usage Examples

### 1. LiveLineChart Component
```svelte
<!-- Greenhouse VPD Chart -->
<LiveLineChart
  title="Greenhouse VPD (60s avg)"
  unit="kPa"
  greenhouseId={1}
  seriesKey="vpd_kpa"
/>

<!-- Node 04 Bag RH2 Chart -->
<LiveLineChart
  title="Node 04 Bag RH2 (60s avg)"
  unit="%"
  greenhouseId={1}
  seriesKey="bag_rh2_pct"
  nodeId={4}
/>

<!-- Outdoor Node PAR Chart -->
<LiveLineChart
  title="Outdoor PAR (60s avg)"
  unit=""
  greenhouseId={1}
  seriesKey="par_value"
  nodeId={65001}
/>
```

### 2. SensorCard Component
```svelte
<!-- Greenhouse Air Temperature -->
<SensorCard
  title="Greenhouse Air Temp"
  unit="°C"
  seriesKey="air_temp_c"
  greenhouseId={1}
  minValue={15}
  maxValue={35}
/>

<!-- Node 01 Leaf Temperature -->
<SensorCard
  title="Node 01 Leaf Temp"
  unit="°C"
  seriesKey="leaf_temp_c"
  greenhouseId={1}
  nodeId={1}
  minValue={20}
  maxValue={40}
/>

<!-- Node 65001 Air Humidity -->
<SensorCard
  title="Outdoor Humidity"
  unit="%"
  seriesKey="air_rh_pct"
  greenhouseId={1}
  nodeId={65001}
  minValue={30}
  maxValue={90}
/>
```

### 3. VerticalGauge Component
```svelte
<!-- Greenhouse Humidity Gauge -->
<VerticalGauge
  title="Greenhouse Humidity"
  unit="%"
  seriesKey="air_rh_pct"
  greenhouseId={1}
  minValue={0}
  maxValue={100}
  redThreshold={20}
  yellowThreshold={80}
/>

<!-- Node 02 Bag RH1 Gauge -->
<VerticalGauge
  title="Node 02 Bag RH1"
  unit="%"
  seriesKey="bag_rh1_pct"
  greenhouseId={1}
  nodeId={2}
  minValue={0}
  maxValue={100}
  redThreshold={30}
  yellowThreshold={85}
/>

<!-- Outdoor Temperature Gauge -->
<VerticalGauge
  title="Outdoor Temperature"
  unit="°C"
  seriesKey="air_temp_c"
  greenhouseId={1}
  nodeId={65001}
  minValue={-20}
  maxValue={50}
  redThreshold={10}
  yellowThreshold={35}
/>
```

### 4. StatusIndicator Component
```svelte
<!-- Rain Sensor Status (if available) -->
<StatusIndicator
  title="Rain Sensor"
  seriesKey="rain_sensor"
  greenhouseId={1}
  nodeId={65001}
  onValue={1}
  offValue={0}
  onColor="#28a745"
  offColor="#dc3545"
/>

<!-- Custom Binary Sensor -->
<StatusIndicator
  title="Pump Status"
  seriesKey="pump_status"
  greenhouseId={1}
  nodeId={1}
  onValue={1}
  offValue={0}
  onColor="#007bff"
  offColor="#6c757d"
/>
```

## Event Streams

### "gh_avg" Events
- **Source**: Greenhouse aggregator
- **Data**: Combined averages from all active standard nodes
- **Use**: Overall greenhouse monitoring
- **nodeId**: Set to `null` or omit

### "node_avg" Events
- **Source**: Individual node aggregators
- **Data**: Per-node 60-second averages
- **Use**: Node-specific monitoring
- **nodeId**: Set to specific node ID (1, 2, 3, 4, or 65001)

## Best Practices

1. **Choose appropriate seriesKey**: Use the sensor type that matches your monitoring needs
2. **Set proper ranges**: Configure minValue/maxValue for accurate color coding
3. **Node selection**: Use greenhouse data for overview, node data for specific locations
4. **Units**: Always specify the correct unit for proper display
5. **Thresholds**: Set meaningful thresholds for gauges and status indicators

## Performance Notes

- All components use efficient event listening
- Data updates every 60 seconds
- Automatic cleanup on component destruction
- Non-blocking message passing from backend
- Minimal memory footprint
