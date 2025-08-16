<script>
  export let buttonName = "Button";
  export let onClick = () => {};
</script>

<button class="cresla-button" on:click={onClick}>
  {buttonName}
</button>

<style>
  .cresla-button {
    min-width: 130px;
    height: 40px;
    color: #fff;
    padding: 5px 10px;
    font-weight: bold;
    cursor: pointer;
    position: relative;
    display: inline-block;
    outline: none;
    border-radius: 10px;
    z-index: 0;
    background: #fff;
    border: 2px solid #000000;
    color: #000000;
    transition:
      background 0.3s,
      color 0.3s,
      transform 0.2s,
      box-shadow 0.2s;
  }
  .cresla-button:hover {
    color: #fff;
    background: #000000;
  }

  .cresla-button:active {
    transform: translateY(2px);
    box-shadow: 2px 2px 3px rgba(0, 0, 0, 0.2);
  }
</style>
