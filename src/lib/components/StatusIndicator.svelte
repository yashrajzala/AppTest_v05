<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { listen } from "@tauri-apps/api/event";

  /**
   * Real-time status indicator for boolean sensors
   * Props:
   * - title: display title for the indicator
   * - seriesKey: which field from GhAvg/NodeAvg payload to display
   * - greenhouseId: filter incoming events by GH id
   * - nodeId: specific node ID (null = greenhouse avg, number = specific node)
   * - onValue: value that represents "on" status (default: 1)
   * - offValue: value that represents "off" status (default: 0)
   * - onColor: color for "on" status (default: green)
   * - offColor: color for "off" status (default: red)
   */
  export let title = "Status";
  export let seriesKey: string = "rain_sensor";
  export let greenhouseId = 1;
  export let nodeId: number | null = null;
  export let onValue: number = 1;
  export let offValue: number = 0;
  export let onColor: string = "#28a745";
  export let offColor: string = "#dc3545";

  let status: "on" | "off" = "off";
  let currentValue: number = 0;
  let unlisten: (() => void) | null = null;
  let nodeUnlisten: (() => void) | null = null;

  function updateStatus(value: number | null) {
    if (value === null || value === undefined) {
      status = "off";
      return;
    }
    
    currentValue = value;
    status = value === onValue ? "on" : "off";
  }

  onMount(async () => {
    // Listen to greenhouse events
    unlisten = await listen("gh_avg", (e: any) => {
      const p = e?.payload;
      if (!p) return;
      if (p.greenhouse_id !== greenhouseId) return;
      if (nodeId !== null) return; // Skip if we want node-specific data
      
      const raw = p[seriesKey] as number | undefined;
      updateStatus(raw === undefined ? null : raw);
    });

    // Listen to node-specific events if nodeId is specified
    if (nodeId !== null) {
      nodeUnlisten = await listen("node_avg", (e: any) => {
        const p = e?.payload;
        if (!p) return;
        if (p.greenhouse_id !== greenhouseId || p.node_id !== nodeId) return;
        
        const raw = p[seriesKey] as number | undefined;
        updateStatus(raw === undefined ? null : raw);
      });
    }
  });

  onDestroy(() => {
    unlisten && unlisten();
    nodeUnlisten && nodeUnlisten();
  });
</script>

<div class="status-container">
  <h4 class="status-title">{title}</h4>
  <div class="indicator {status}" style="--on-color: {onColor}; --off-color: {offColor};"></div>
  <span class="status-text">{status.toUpperCase()}</span>
</div>

<style>
  .status-container {
    display: flex;
    flex-direction: column;
    align-items: center;
    gap: 8px;
    padding: 16px;
    border-radius: 10px;
    background-color: #fff;
    box-shadow: 0 4px 15px rgba(0, 0, 0, 0.08);
    min-width: 80px;
  }

  .status-title {
    font-size: 0.875rem;
    font-weight: 500;
    color: #333;
    margin: 0;
    text-align: center;
  }

  .indicator {
    width: 20px;
    height: 20px;
    border-radius: 50%;
    box-shadow: 0 0 5px rgba(0, 0, 0, 0.2);
    animation: pulse 1.5s infinite;
  }

  /* Green for on status */
  .indicator.on {
    background-color: var(--on-color);
    box-shadow: 0 0 10px var(--on-color);
  }

  /* Red for off status */
  .indicator.off {
    background-color: var(--off-color);
    box-shadow: 0 0 10px var(--off-color);
  }

  .status-text {
    font-size: 0.75rem;
    font-weight: 600;
    color: #666;
    text-transform: uppercase;
    letter-spacing: 0.5px;
  }

  @keyframes pulse {
    0% {
      transform: scale(1);
    }
    50% {
      transform: scale(1.1);
    }
    100% {
      transform: scale(1);
    }
  }
</style>
