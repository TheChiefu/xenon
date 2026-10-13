<script lang="ts">
  import Steam from "@/components/settings/steam.svelte";
  import Xbox from "@/components/settings/xbox.svelte";
</script>

<style>

  .links :global {

    hr {
      border-color: var(--component-raised);
    }

    .platform {
      display: flex;
      flex-direction: column;
      gap: 0.5rem;
    }

    .title {
      font-size: 1.1rem;
      font-weight: 500;
    }

    .platform-row {
      display: flex;
      align-items: center;
      gap: 0.5rem;
    }

    .platform-text {
      display: flex;
      align-items: baseline;
      gap: 0.5rem;
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
      gap: 0.5rem;
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
