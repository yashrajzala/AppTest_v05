# Greenhouse Sensor Dashboard Components

## Overview
This project provides a comprehensive set of real-time sensor monitoring components for greenhouse environments. All components now support automatic data binding through `seriesKey` and `nodeId` props, eliminating the need for manual data management.

## Refactored Components

### 🎯 **LiveLineChart** - Real-time Line Charts
**Purpose**: Display time-series data with interactive tooltips and responsive design.

**Key Features**:
- Real-time 60-second data updates
- Interactive tooltips with local time
- Responsive sizing (3 charts per row on 1080p, 1 on mobile)
- Support for both greenhouse and node-specific data

**Props**:
- `title`: Chart display title
- `unit`: Measurement unit (e.g., "°C", "%", "kPa")
- `seriesKey`: Sensor field to monitor
- `greenhouseId`: Target greenhouse ID
- `nodeId`: Specific node ID (null = greenhouse avg, number = specific node)
- `maxPoints`: Maximum data points to keep in memory

### 📊 **SensorCard** - Value Display Cards
**Purpose**: Show current sensor values with automatic color coding based on thresholds.

**Key Features**:
- Real-time value updates
- Automatic color coding (red/yellow/green based on value ranges)
- Clean, card-based design matching your existing UI
- Support for all sensor types

**Props**:
- `title`: Display title
- `unit`: Measurement unit
- `seriesKey`: Sensor field to monitor
- `greenhouseId`: Target greenhouse ID
- `nodeId`: Specific node ID
- `valueColor`: Manual color override (optional)
- `minValue`/`maxValue`: Range for automatic color calculation

### 📈 **VerticalGauge** - Visual Gauge Displays
**Purpose**: Provide visual gauge representation with color-coded thresholds.

**Key Features**:
- Vertical gauge with scale markers
- Configurable color thresholds (red/yellow/green)
- Real-time value updates
- Smooth animations

**Props**:
- `title`: Gauge title
- `unit`: Measurement unit
- `seriesKey`: Sensor field to monitor
- `greenhouseId`: Target greenhouse ID
- `nodeId`: Specific node ID
- `minValue`/`maxValue`: Gauge scale range
- `redThreshold`/`yellowThreshold`: Color threshold percentages

### 🔴 **StatusIndicator** - Binary Status Display
**Purpose**: Show on/off status for binary sensors (e.g., rain sensors, pumps).

**Key Features**:
- Binary status display (on/off)
- Configurable on/off values and colors
- Animated pulse effect
- Support for custom binary sensors

**Props**:
- `title`: Status title
- `seriesKey`: Sensor field to monitor
- `greenhouseId`: Target greenhouse ID
- `nodeId`: Specific node ID
- `onValue`/`offValue`: Values representing on/off states
- `onColor`/`offColor`: Colors for on/off states

## Quick Start

### 1. Basic Greenhouse Monitoring
```svelte
<script>
  import LiveLineChart from "$lib/components/LiveLineChart.svelte";
  import SensorCard from "$lib/components/SensorCard.svelte";
</script>

<!-- Greenhouse VPD Chart -->
<LiveLineChart
  title="Greenhouse VPD"
  unit="kPa"
  greenhouseId={1}
  seriesKey="vpd_kpa"
/>

<!-- Greenhouse Temperature Card -->
<SensorCard
  title="Air Temperature"
  unit="°C"
  seriesKey="air_temp_c"
  greenhouseId={1}
  minValue={15}
  maxValue={35}
/>
```

### 2. Node-Specific Monitoring
```svelte
<!-- Node 04 Bag RH2 Chart -->
<LiveLineChart
  title="Node 04 Bag RH2"
  unit="%"
  greenhouseId={1}
  seriesKey="bag_rh2_pct"
  nodeId={4}
/>

<!-- Node 01 Leaf Temperature Gauge -->
<VerticalGauge
  title="Node 01 Leaf Temp"
  unit="°C"
  seriesKey="leaf_temp_c"
  greenhouseId={1}
  nodeId={1}
  minValue={20}
  maxValue={40}
  redThreshold={25}
  yellowThreshold={35}
/>
```

### 3. Outdoor Monitoring
```svelte
<!-- Outdoor PAR Chart -->
<LiveLineChart
  title="Outdoor PAR"
  unit=""
  greenhouseId={1}
  seriesKey="par_value"
  nodeId={65001}
/>

<!-- Outdoor Temperature Card -->
<SensorCard
  title="Outdoor Temp"
  unit="°C"
  seriesKey="air_temp_c"
  greenhouseId={1}
  nodeId={65001}
  minValue={-20}
  maxValue={50}
/>
```

## Data Sources

### Event Streams
- **`"gh_avg"`**: Greenhouse-level 60-second averages
- **`"node_avg"`**: Node-specific 60-second averages

### Available Sensors
- **Temperature**: `air_temp_c`, `leaf_temp_c`, `bag_temp_c`
- **Humidity**: `air_rh_pct`, `bag_rh1_pct`, `bag_rh2_pct`, `bag_rh3_pct`, `bag_rh4_pct`, `bag_rh_avg_pct`
- **Environmental**: `par_value`, `weight_g`
- **Pressure**: `ea_air_kpa`, `ea_leaf_kpa`, `es_kpa`, `vpd_kpa`

### Node Types
- **Standard Nodes (01-04)**: Full sensor suite, inside greenhouse
- **Outdoor Node (65001)**: Limited sensors, outside greenhouse
- **Greenhouse Aggregator**: Combined averages from all standard nodes

## Architecture Benefits

### 🚀 **Performance**
- Efficient event listening with automatic cleanup
- Non-blocking message passing from Rust backend
- Minimal memory footprint
- 60-second update intervals

### 🛡️ **Reliability**
- Automatic error handling for missing data
- Graceful fallbacks for undefined values
- Proper cleanup on component destruction
- Type-safe TypeScript implementation

### 🏭 **Modularity**
- Consistent API across all components
- Reusable props and event handling
- Easy to extend with new sensor types
- Clean separation of concerns

### 🎨 **User Experience**
- Real-time updates without page refresh
- Responsive design for all screen sizes
- Interactive elements with tooltips
- Consistent visual styling

## Best Practices

1. **Choose appropriate seriesKey**: Match sensor type to monitoring needs
2. **Set meaningful ranges**: Configure minValue/maxValue for accurate color coding
3. **Use nodeId wisely**: Greenhouse data for overview, node data for specific locations
4. **Specify units**: Always include proper measurement units
5. **Configure thresholds**: Set meaningful thresholds for gauges and status indicators

## Troubleshooting

### No Data Displaying
- Check `seriesKey` matches available sensor fields
- Verify `greenhouseId` is correct
- Ensure `nodeId` is set correctly (null for greenhouse, number for nodes)
- Check backend is emitting events

### Performance Issues
- Components automatically clean up listeners
- Data updates every 60 seconds (backend controlled)
- Memory usage is minimal and bounded

## Contributing

When adding new components:
1. Follow the established pattern of `seriesKey` + `nodeId` props
2. Implement proper cleanup in `onDestroy`
3. Use TypeScript for type safety
4. Maintain consistent styling with existing components
5. Add comprehensive documentation and examples

## Support

For detailed SeriesKey reference, see [VARIABLES.md](./VARIABLES.md).

For component-specific examples and usage patterns, refer to the individual component files in `src/lib/components/`.
