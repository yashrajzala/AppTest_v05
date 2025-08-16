<script lang="ts">
  import LiveLineChart from "$lib/components/LiveLineChart.svelte";
  import SensorCard from "$lib/components/SensorCard.svelte";
  import VerticalGauge from "$lib/components/VerticalGauge.svelte";
  import StatusIndicator from "$lib/components/StatusIndicator.svelte";
</script>

<svelte:head>
  <title>Greenhouse Sensor Dashboard</title>
</svelte:head>

<main>
  <h1>Greenhouse Sensor Dashboard</h1>
  
  <!-- LiveLineChart Examples -->
  <section>
    <h2>Real-time Charts</h2>
    <div class="charts-grid">
      <!-- Greenhouse-level charts -->
      <LiveLineChart
        title="Greenhouse VPD (60s avg)"
        unit="kPa"
        greenhouseId={1}
        seriesKey="vpd_kpa"
        maxPoints={1440}
      />
      
      <LiveLineChart
        title="Greenhouse Air Temp (60s avg)"
        unit="°C"
        greenhouseId={1}
        seriesKey="air_temp_c"
        maxPoints={1440}
      />
      
      <LiveLineChart
        title="Greenhouse Humidity (60s avg)"
        unit="%"
        greenhouseId={1}
        seriesKey="air_rh_pct"
        maxPoints={1440}
      />
      
      <LiveLineChart
        title="Greenhouse PAR (60s avg)"
        unit=""
        greenhouseId={1}
        seriesKey="par_value"
        maxPoints={1440}
      />
    </div>
  </section>

  <!-- Node-specific charts -->
  <section>
    <h2>Node-Specific Charts</h2>
    <div class="charts-grid">
      <LiveLineChart
        title="Node 04 Bag RH2 (60s avg)"
        unit="%"
        greenhouseId={1}
        seriesKey="bag_rh2_pct"
        nodeId={4}
        maxPoints={1440}
      />
      
      <LiveLineChart
        title="Outdoor Node PAR (60s avg)"
        unit=""
        greenhouseId={1}
        seriesKey="par_value"
        nodeId={65001}
        maxPoints={1440}
      />
      
      <LiveLineChart
        title="Node 01 Air Temp (60s avg)"
        unit="°C"
        greenhouseId={1}
        seriesKey="air_temp_c"
        nodeId={1}
        maxPoints={1440}
      />
      
      <LiveLineChart
        title="Node 02 Leaf Temp (60s avg)"
        unit="°C"
        greenhouseId={1}
        seriesKey="leaf_temp_c"
        nodeId={2}
        maxPoints={1440}
      />
    </div>
  </section>

  <!-- SensorCard Examples -->
  <section>
    <h2>Current Values</h2>
    <div class="cards-grid">
      <SensorCard
        title="Greenhouse Air Temp"
        unit="°C"
        seriesKey="air_temp_c"
        greenhouseId={1}
        minValue={15}
        maxValue={35}
      />
      
      <SensorCard
        title="Node 01 Leaf Temp"
        unit="°C"
        seriesKey="leaf_temp_c"
        greenhouseId={1}
        nodeId={1}
        minValue={20}
        maxValue={40}
      />
      
      <SensorCard
        title="Outdoor Humidity"
        unit="%"
        seriesKey="air_rh_pct"
        greenhouseId={1}
        nodeId={65001}
        minValue={30}
        maxValue={90}
      />
      
      <SensorCard
        title="Greenhouse VPD"
        unit="kPa"
        seriesKey="vpd_kpa"
        greenhouseId={1}
        minValue={0}
        maxValue={5}
      />
    </div>
  </section>

  <!-- VerticalGauge Examples -->
  <section>
    <h2>Gauge Displays</h2>
    <div class="gauges-grid">
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
      
      <VerticalGauge
        title="Greenhouse VPD"
        unit="kPa"
        seriesKey="vpd_kpa"
        greenhouseId={1}
        minValue={0}
        maxValue={5}
        redThreshold={20}
        yellowThreshold={80}
      />
    </div>
  </section>

  <!-- StatusIndicator Examples -->
  <section>
    <h2>Status Indicators</h2>
    <div class="status-grid">
      <!-- Example for future binary sensors -->
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
      
      <StatusIndicator
        title="Fan Status"
        seriesKey="fan_status"
        greenhouseId={1}
        nodeId={2}
        onValue={1}
        offValue={0}
        onColor="#ffc107"
        offColor="#6c757d"
      />
      
      <StatusIndicator
        title="Light Status"
        seriesKey="light_status"
        greenhouseId={1}
        nodeId={3}
        onValue={1}
        offValue={0}
        onColor="#fd7e14"
        offColor="#6c757d"
      />
    </div>
  </section>
</main>

<style>
  main {
    max-width: 1400px;
    margin: 0 auto;
    padding: 20px;
  }

  h1 {
    text-align: center;
    color: #2b2b2b;
    margin-bottom: 40px;
  }

  h2 {
    color: #333;
    margin: 40px 0 20px 0;
    border-bottom: 2px solid #e0e0e0;
    padding-bottom: 10px;
  }

  .charts-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(600px, 1fr));
    gap: 20px;
    margin-bottom: 40px;
  }

  .cards-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    gap: 20px;
    margin-bottom: 40px;
  }

  .gauges-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(250px, 1fr));
    gap: 20px;
    margin-bottom: 40px;
  }

  .status-grid {
    display: grid;
    grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
    gap: 20px;
    margin-bottom: 40px;
  }

  @media (max-width: 768px) {
    .charts-grid {
      grid-template-columns: 1fr;
    }
    
    .cards-grid {
      grid-template-columns: repeat(auto-fit, minmax(150px, 1fr));
    }
    
    .gauges-grid {
      grid-template-columns: repeat(auto-fit, minmax(200px, 1fr));
    }
    
    .status-grid {
      grid-template-columns: repeat(auto-fit, minmax(120px, 1fr));
    }
  }
</style>
