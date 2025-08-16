<script lang="ts">
  import uPlot from "uplot";
  import "uplot/dist/uPlot.min.css";
  import { onMount, onDestroy } from "svelte";
  import { listen } from "@tauri-apps/api/event";

  /**
   * Reusable live line chart (uPlot)
   * Props:
   * - title: chart title
   * - unit: y-axis unit label
   * - maxPoints: max points kept in RAM (default 1440 = 24h @ 1/min)
   * - greenhouseId: filter incoming events by GH id
   * - seriesKey: which field from GhAvg/NodeAvg payload to plot (e.g., "vpd_kpa")
   * - nodeId: specific node ID (null = greenhouse avg, number = specific node)
   *
   * Available seriesKey values:
   * 
   * From "gh_avg" event (60s greenhouse averages):
   * - Temperature: air_temp_c, leaf_temp_c, bag_temp_c
   * - Humidity: air_rh_pct, bag_rh1_pct, bag_rh2_pct, bag_rh3_pct, bag_rh4_pct, bag_rh_avg_pct
   * - Environmental: par_value, weight_g
   * - Pressure: ea_air_kpa, ea_leaf_kpa, es_kpa, vpd_kpa
   * 
   * From "node_avg" event (60s per-node averages):
   * - Same fields as above + ts_ms, greenhouse_id, node_id
   * - Nodes: 01, 02, 03, 04 (Standard), 65001 (Outdoor)
   *
   * CHART EXAMPLES:
   * 
   * 1. Greenhouse-level charts (listen to "gh_avg"):
   *    <LiveLineChart
   *      title="Greenhouse VPD (60s avg)"
   *      unit="kPa"
   *      greenhouseId={1}
   *      seriesKey="vpd_kpa"
   *      // nodeId not set = listens to gh_avg events
   *    />
   *
   * 2. Node-specific charts (listen to "node_avg"):
   *    <LiveLineChart
   *      title="Node 04 Bag RH2 (60s avg)"
   *      unit="%"
   *      greenhouseId={1}
   *      seriesKey="bag_rh2_pct"
   *      nodeId={4}  // Filters node_avg events for node 04
   *    />
   *
   *    <LiveLineChart
   *      title="Outdoor Node PAR (60s avg)"
   *      unit=""
   *      greenhouseId={1}
   *      seriesKey="par_value"
   *      nodeId={65001}  // Filters node_avg events for outdoor node
   *    />
   *
   * 3. Node comparison charts:
   *    <LiveLineChart
   *      title="All Nodes Air Temp (60s avg)"
   *      unit="°C"
   *      greenhouseId={1}
   *      seriesKey="air_temp_c"
   *      // No nodeId = shows greenhouse average (gh_avg)
   *    />
   *
   * HOW IT WORKS:
   * - nodeId = null: Listens to "gh_avg" events (greenhouse averages)
   * - nodeId = number: Listens to "node_avg" events filtered by specific node_id
   * - Both use same seriesKey fields but from different event streams
   */
  export let title = "Live Sensor";
  export let unit = "";
  export let maxPoints = 1440;
  export let greenhouseId = 1;
  export let seriesKey: string = "vpd_kpa";
  export let nodeId: number | null = null; // null = all nodes, specific number = filter by node

  // Data buffers
  let x: number[] = [];
  let y: number[] = [];

  let u: uPlot | null = null;
  let rootEl: HTMLDivElement;
  let containerEl: HTMLDivElement;
  let tooltipEl: HTMLDivElement | null = null;

  // Responsive sizing: up to 3 charts on 1080p, single on mobile
  const maxChartWidth = 640;
  const minChartWidth = 300;
  const chartHeight = 280;

  const opts: uPlot.Options = {
    width: maxChartWidth,
    height: chartHeight,
    title,
    pxAlign: 1,
    scales: {
      x: { time: true },
      y: { auto: true },
    },
    axes: [
      {
        stroke: "#999",
        grid: { show: true },
        values: (u, vals) => vals.map((sec) => new Date((sec as number) * 1000).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" })),
      },
      {
        stroke: "#999",
        grid: { show: true },
        label: unit ? `${unit}` : "",
      },
    ],
    series: [
      {}, // x
      {
        label: seriesKey,
        spanGaps: true,
        width: 2,
        points: {
          show: true,
          size: 5,
          stroke: "#2b8a3e",
          fill: "#2b8a3e",
        },
        stroke: "#2b8a3e",
      },
    ],
    cursor: { focus: { prox: 16 } },
    legend: { show: true },
    hooks: {
      setCursor: [
        (chart) => {
          const idx = chart.cursor.idx;
          const left = chart.cursor.left;
          const top = chart.cursor.top;
          if (idx == null || left == null || top == null) {
            if (tooltipEl) tooltipEl.style.display = "none";
            return;
          }
          const ts = chart.data[0][idx] as number;
          const val = chart.data[1][idx] as number;
          if (ts == null || val == null) return;

          if (!tooltipEl) {
            tooltipEl = document.createElement("div");
            tooltipEl.style.position = "fixed";
            tooltipEl.style.pointerEvents = "none";
            tooltipEl.style.background = "rgba(0,0,0,0.85)";
            tooltipEl.style.color = "#fff";
            tooltipEl.style.fontSize = "12px";
            tooltipEl.style.padding = "6px 8px";
            tooltipEl.style.borderRadius = "6px";
            tooltipEl.style.zIndex = "1000";
            document.body.appendChild(tooltipEl);
          }

          const timeStr = new Date(ts * 1000).toLocaleTimeString([], { hour: "2-digit", minute: "2-digit" });
          tooltipEl.textContent = `${timeStr}  ${val}${unit ? ` ${unit}` : ""}`;

          const rect = containerEl.getBoundingClientRect();
          const xPos = Math.min(rect.left + left + 10, window.innerWidth - 80);
          const yPos = Math.max(rect.top + top - 30, 10);
          tooltipEl.style.left = `${xPos}px`;
          tooltipEl.style.top = `${yPos}px`;
          tooltipEl.style.display = "block";
        },
      ],
    },
  };

  function pushPoint(ts: number, val: number | null) {
    if (val == null || Number.isNaN(val)) return;
    const tsSec = Math.floor(ts / 1000);
    const lastTs = x.length ? x[x.length - 1] : 0;
    if (tsSec === lastTs) {
      y[y.length - 1] = val;
    } else {
      x.push(tsSec);
      y.push(val);
    }
    const over = x.length - maxPoints;
    if (over > 0) {
      x.splice(0, over);
      y.splice(0, over);
    }
    u && u.setData([x, y]);
  }

  let unlisten: (() => void) | null = null;
  let nodeUnlisten: (() => void) | null = null;
  let resizeObserver: ResizeObserver | null = null;

  function handleResize() {
    if (!containerEl || !u) return;
    const containerWidth = containerEl.clientWidth;
    const availableWidth = Math.min(containerWidth, maxChartWidth);
    const chartWidth = Math.max(availableWidth, minChartWidth);
    if (u.width !== chartWidth) {
      u.setSize({ width: chartWidth, height: chartHeight });
    }
  }

  onMount(async () => {
    u = new uPlot(opts, [x, y], rootEl);

    resizeObserver = new ResizeObserver(handleResize);
    if (containerEl) {
      resizeObserver.observe(containerEl);
      handleResize();
    }
    window.addEventListener("resize", handleResize);

    // Listen to both greenhouse and node events
    unlisten = await listen("gh_avg", (e: any) => {
      const p = e?.payload;
      if (!p) return;
      if (p.greenhouse_id !== greenhouseId) return;
      const raw = p[seriesKey] as number | undefined;
      if (raw === undefined || raw === null) return;
      const v = Math.round(raw * 100) / 100;
      pushPoint(p.ts_ms as number, v);
    });

    // Listen to node-specific events if nodeId is specified
    if (nodeId !== null) {
      nodeUnlisten = await listen("node_avg", (e: any) => {
        const p = e?.payload;
        if (!p) return;
        if (p.greenhouse_id !== greenhouseId || p.node_id !== nodeId) return;
        const raw = p[seriesKey] as number | undefined;
        if (raw === undefined || raw === null) return;
        const v = Math.round(raw * 100) / 100;
        pushPoint(p.ts_ms as number, v);
      });
    }
  });

  onDestroy(() => {
    unlisten && unlisten();
    nodeUnlisten && nodeUnlisten();
    if (u) {
      u.destroy();
      u = null;
    }
    if (resizeObserver) {
      resizeObserver.disconnect();
      resizeObserver = null;
    }
    window.removeEventListener("resize", handleResize);
    if (tooltipEl) {
      tooltipEl.remove();
      tooltipEl = null;
    }
  });
</script>

<div class="chart-container" bind:this={containerEl}>
  <div class="chart-card">
    <div bind:this={rootEl} class="chart-wrap"></div>
  </div>
</div>

<style>
  .chart-container {
    width: 100%;
    max-width: 100%;
    display: flex;
    justify-content: center;
    align-items: center;
  }

  .chart-card {
    min-width: 300px;
    max-width: 640px;
    padding: 20px;
    border-radius: 10px;
    background-color: #fff;
    box-shadow: 0 4px 15px rgba(0, 0, 0, 0.08);
    width: 100%;
  }

  .chart-wrap {
    width: 100%;
    height: 100%;
    display: flex;
    justify-content: center;
    align-items: center;
  }

  :global(.u-title) {
    font-weight: 600;
    color: #2b2b2b;
  }

  :global(.u-legend) {
    font-size: 12px;
  }

  @media (max-width: 768px) {
    .chart-card {
      max-width: 100%;
      min-width: 250px;
    }
  }
</style>
