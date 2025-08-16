<script lang="ts">
	import { onMount, onDestroy } from "svelte";
	import { listen } from "@tauri-apps/api/event";

	/**
	 * Real-time vertical gauge component
	 * Props:
	 * - title: display title for the gauge
	 * - unit: unit of measurement (e.g., "°C", "%", "kPa")
	 * - seriesKey: which field from GhAvg/NodeAvg payload to display
	 * - greenhouseId: filter incoming events by GH id
	 * - nodeId: specific node ID (null = greenhouse avg, number = specific node)
	 * - minValue: minimum value for gauge scale and color calculation
	 * - maxValue: maximum value for gauge scale and color calculation
	 * - redThreshold: percentage threshold for red color (0-100)
	 * - yellowThreshold: percentage threshold for yellow color (0-100)
	 */
	export let title: string = 'Gauge';
	export let unit: string = '%';
	export let seriesKey: string = "vpd_kpa";
	export let greenhouseId = 1;
	export let nodeId: number | null = null;
	export let minValue: number = 0;
	export let maxValue: number = 100;
	export let redThreshold: number = 20; // Default: Red from 0% to 20%
	export let yellowThreshold: number = 80; // Default: Yellow from 21% to 80%

	let currentValue: number = 0;
	let unlisten: (() => void) | null = null;
	let nodeUnlisten: (() => void) | null = null;

	// Reactive variable to calculate the gauge fill percentage
	$: fillPercentage = (Math.max(0, currentValue - minValue) / (maxValue - minValue)) * 100;

	// Determine the fill color based on the percentage and user-defined thresholds
	$: {
		if (fillPercentage <= redThreshold) {
			// Danger level (red)
			fillColor = 'hsl(0, 70%, 50%)';
		} else if (fillPercentage <= yellowThreshold) {
			// Warning level (yellow)
			fillColor = 'hsl(50, 70%, 50%)';
		} else {
			// Normal level (green)
			fillColor = 'hsl(120, 70%, 50%)';
		}
	}
	let fillColor: string;

	function updateValue(value: number | null) {
		if (value === null || value === undefined) {
			currentValue = minValue;
			return;
		}
		currentValue = value;
	}

	/**
	 * Generates the scale markers for the vertical axis.
	 * @returns An array of numbers representing the scale markers.
	 */
	function getScaleMarkers(): number[] {
		const markers: number[] = [];
		const step = (maxValue - minValue) / 10;
		for (let i = 0; i <= 10; i++) {
			const value = Math.round(minValue + step * i);
			markers.push(value);
		}
		return markers;
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

<div class="gauge-container">
	<h2 class="gauge-title">{title}</h2>

	<div class="gauge-wrapper">
		<!-- The vertical scale marks -->
		<div class="gauge-scale">
			{#each getScaleMarkers() as marker}
				<span class="scale-marker">{marker}</span>
			{/each}
		</div>

		<!-- The main gauge body -->
		<div class="gauge-body">
			<!-- The gauge markings (lines) -->
			{#each getScaleMarkers() as marker}
				<div
					class="gauge-marking"
					style="bottom: {(Math.max(0, marker - minValue) / (maxValue - minValue)) * 100}%;"
				></div>
			{/each}

			<!-- The colored fill level, dynamically sized and colored -->
			<div
				class="gauge-fill"
				style="height: {fillPercentage}%; background-color: {fillColor};"
			></div>
		</div>
	</div>

	<!-- Display the current value and unit -->
	<div class="gauge-value">
		<span class="value-number">{currentValue.toFixed(0)}</span>
		<span class="value-unit">{unit}</span>
	</div>
</div>

<style>
	.gauge-container {
		display: flex;
		flex-direction: column;
		align-items: center;
		padding: 20px;
		border-radius: 10px;
		box-shadow: 0 4px 6px rgba(0, 0, 0, 0.1);
		background-color: #f9f9f9;
		font-family: Arial, sans-serif;
	}

	.gauge-title {
		font-size: 1.25rem;
		font-weight: 600;
		color: #333;
		margin-bottom: 1rem;
	}

	.gauge-wrapper {
		display: flex;
		align-items: stretch;
		height: 320px;
	}

	.gauge-scale {
		display: flex;
		flex-direction: column-reverse;
		justify-content: space-between;
		text-align: right;
		font-size: 0.875rem;
		color: #666;
		padding-right: 8px;
		flex-grow: 0;
		flex-shrink: 0;
	}

	.scale-marker {
		position: relative;
		transform: translateY(50%);
	}

	.gauge-body {
		position: relative;
		width: 64px;
		height: 100%;
		background-color: #e0e0e0;
		border-radius: 10px;
		box-shadow: inset 0 2px 4px rgba(0, 0, 0, 0.06);
		overflow: hidden;
	}

	.gauge-marking {
		position: absolute;
		left: 0;
		right: 0;
		height: 2px;
		background-color: #888;
		z-index: 2; /* Ensure markers are on top of the fill */
	}

	.gauge-fill {
		position: absolute;
		bottom: 0;
		width: 100%;
		border-radius: 10px;
		transition: height 0.3s ease-in-out;
		z-index: 1; /* Ensure fill is below the markers */
	}

	.gauge-value {
		display: flex;
		flex-direction: column;
		align-items: center;
		margin-top: 1rem;
		color: #555;
	}

	.value-number {
		font-size: 2rem;
		font-weight: 700;
	}

	.value-unit {
		font-size: 0.875rem;
		color: #888;
	}
</style>
