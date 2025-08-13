<script lang="ts">
  import LiveLineChart from "$lib/components/LiveLineChart.svelte";
</script>

<section style="padding: 12px;">
  <!-- set GH ID as needed -->
  <LiveLineChart
    title="Greenhouse VPD (avg, 60s)"
    unit="kPa"
    greenhouseId={1}
    seriesKey="vpd_kpa"
    maxPoints={1440}
  />
</section>
