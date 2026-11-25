<script lang="ts">
  import Steam from "./steam.svelte";
  import Xbox from "./xbox.svelte";
</script>

<style>

  .links :global {

    .platform {
      display: flex;
      flex-direction: column;
      gap: var(--row-gap);
    }

    .title {
      font-size: 1.1rem;
      font-weight: 500;
    }

    .platform-row {
      display: flex;
      align-items: center;
      gap: var(--inline-gap);
    }

    .platform-text {
      display: flex;
      align-items: baseline;
      gap: var(--inline-gap);
    }

    .platform p {
      margin: 0;
    }

    .platform .message {
      margin-left: auto;
    }

    .platform-text,
    .platform-row label {
      flex: 1;
    }

    .platform-buttons {
      display: flex;
      gap: var(--inline-gap);
    }

    .handle {
      font-style: italic;
      color: var(--text-dim);
      cursor: default;
      user-select: none;
    }
  }
</style>

<div class="links">
  <Xbox/>
  <Steam/>
</div>
