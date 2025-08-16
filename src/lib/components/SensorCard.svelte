<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { listen } from "@tauri-apps/api/event";

  /**
   * Real-time sensor value display card
   * Props:
   * - title: display title for the sensor
   * - unit: unit of measurement (e.g., "°C", "%", "kPa")
   * - seriesKey: which field from GhAvg/NodeAvg payload to display
   * - greenhouseId: filter incoming events by GH id
   * - nodeId: specific node ID (null = greenhouse avg, number = specific node)
   * - valueColor: color for the value display (optional, auto-calculated if not set)
   * - minValue: minimum value for color calculation (optional)
   * - maxValue: maximum value for color calculation (optional)
   */
  export let title = "Sensor Name";
  export let unit = "";
  export let seriesKey: string = "vpd_kpa";
  export let greenhouseId = 1;
  export let nodeId: number | null = null;
  export let valueColor: string | null = null;
  export let minValue: number | null = null;
  export let maxValue: number | null = null;

  let sensorValue: string = "XX";
  let currentValue: number = 0;
  let unlisten: (() => void) | null = null;
  let nodeUnlisten: (() => void) | null = null;

  // Auto-calculate color based on value range if not specified
  $: displayColor = valueColor || getAutoColor(currentValue, minValue, maxValue);

  function getAutoColor(value: number, min: number | null, max: number | null): string {
    if (min === null || max === null) return "#000000";
    
    const percentage = (value - min) / (max - min);
    if (percentage <= 0.2) return "#dc3545"; // Red for low values
    if (percentage <= 0.8) return "#ffc107"; // Yellow for medium values
    return "#28a745"; // Green for high values
  }

  function updateValue(value: number | null) {
    if (value === null || value === undefined) {
      sensorValue = "N/A";
      currentValue = 0;
      return;
    }
    
    currentValue = value;
    // Format based on unit type
    if (unit === "°C" || unit === "%" || unit === "kPa") {
      sensorValue = value.toFixed(1);
    } else if (unit === "") {
      sensorValue = value.toFixed(0);
    } else {
      sensorValue = value.toFixed(2);
    }
  }

  onMount(async () => {
    // Listen to greenhouse events
    unlisten = await listen("gh_avg", (e: any) => {
      const p = e?.payload;
      if (!p) return;
      if (p.greenhouse_id !== greenhouseId) return;
      if (nodeId !== null) return; // Skip if we want node-specific data
      
      const raw = p[seriesKey] as number | undefined;
      updateValue(raw === undefined ? null : raw);
    });

    // Listen to node-specific events if nodeId is specified
    if (nodeId !== null) {
      nodeUnlisten = await listen("node_avg", (e: any) => {
        const p = e?.payload;
        if (!p) return;
        if (p.greenhouse_id !== greenhouseId || p.node_id !== nodeId) return;
        
        const raw = p[seriesKey] as number | undefined;
        updateValue(raw === undefined ? null : raw);
      });
    }
  });

  onDestroy(() => {
    unlisten && unlisten();
    nodeUnlisten && nodeUnlisten();
  });
</script>

<div class="sensor-card">
  <h3 class="sensor-name">{title}</h3>
  <p class="sensor-value" style="color: {displayColor};">{sensorValue}{unit}</p>
</div>

<style>
  .sensor-card {
    min-width: 50px;
    padding: 25px;
    border-radius: 10px;
    background-color: #fff;
    box-shadow: 0 4px 15px rgba(0, 0, 0, 0.08);
    text-align: center;
    width: fit-content;
  }

  .sensor-card .sensor-name {
    font-size: 1rem;
    font-weight: 400;
    color: #000000;
    margin: 0 0 15px 0;
  }

  .sensor-card .sensor-value {
    font-size: 1.5rem;
    font-weight: 500;
    margin: 0;
  }
</style>
