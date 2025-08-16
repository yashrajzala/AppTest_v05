<div class="flexible-row-card">
  <slot />
</div>

<style>
  .flexible-row-card {
    display: flex;
    flex-wrap: wrap;
    gap: 20px;
    padding: 20px;
    background-color: #f8f9fa;
    border-radius: 12px;
    box-shadow: 0 4px 8px rgba(0, 0, 0, 0.05);
    justify-content: flex-start;
    align-items: center;
    width: fit-content;
  }
</style>
