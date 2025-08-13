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
   * - seriesKey: which field from GhAvg payload to plot (e.g., "vpd_kpa")
   */
  export let title = "Live Sensor";
  export let unit = "";
  export let maxPoints = 1440;
  export let greenhouseId = 1;
  export let seriesKey: string = "vpd_kpa";

  // Data buffers
  let x: number[] = [];
  let y: number[] = [];

  let u: uPlot | null = null;
  let rootEl: HTMLDivElement;

  const opts: uPlot.Options = {
    width: 800,
    height: 280,
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
        values: (u, vals) =>
          vals.map((v) =>
            new Date(v).toLocaleTimeString([], {
              hour: "2-digit",
              minute: "2-digit",
            })
          ),
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
        points: { show: false },
        stroke: "#2b8a3e",
      },
    ],
    cursor: { focus: { prox: 16 } },
    legend: { show: true },
  };

  function pushPoint(ts: number, val: number | null) {
    if (val == null || Number.isNaN(val)) return;
    const lastTs = x.length ? x[x.length - 1] : 0;
    if (ts === lastTs) {
      y[y.length - 1] = val;
    } else {
      x.push(ts);
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

  onMount(async () => {
    u = new uPlot(opts, [x, y], rootEl);

    // Expect full GhAvg payload from backend:
    // { ts_ms, greenhouse_id, ..., vpd_kpa, ... }
    unlisten = await listen("gh_avg", (e: any) => {
      const p = e?.payload;
      if (!p) return;
      if (p.greenhouse_id !== greenhouseId) return;

      const raw = p[seriesKey] as number | undefined;
      if (raw === undefined || raw === null) return;

      const v = Math.round(raw * 100) / 100; // 2 decimals
      pushPoint(p.ts_ms as number, v);
    });
  });

  onDestroy(() => {
    unlisten && unlisten();
    if (u) {
      u.destroy();
      u = null;
    }
  });
</script>

<div bind:this={rootEl} class="chart-wrap"></div>

<style>
  .chart-wrap {
    width: 100%;
    max-width: 980px;
    margin: 0.5rem auto;
  }
  :global(.u-title) {
    font-weight: 600;
    color: #2b2b2b;
  }
  :global(.u-legend) {
    font-size: 12px;
  }
</style>
