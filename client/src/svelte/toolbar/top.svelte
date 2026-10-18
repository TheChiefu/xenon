<script lang="ts">
  import Identity from "./identity.svelte";
  import ServerInfo from "./server_info.svelte";
  import { Pane } from "@/lib/display/panes.svelte";
  import type { ToolbarButton } from "@/lib/display/toolbar";

  interface Props {
    buttons: ToolbarButton[];
  }

  let { buttons }: Props = $props();
</script>

<style>
  .toolbar {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 0;
    border-bottom: 1px solid var(--border);
    background: var(--component);
  }

  .spacer {
    flex: 1;
  }

  .toolbar-user {
    height: 100%;
  }

  .pane-btn {
    opacity: 0.4;
  }

  .pane-btn.open {
    opacity: 1;
  }

</style>

<!-- Pane Toggle -->
{#snippet toggle(button: ToolbarButton)}
  <button
    class="icon-btn pane-btn"
    class:open={button.active}
    aria-label={button.label}
    title={button.label}
    onclick={button.onclick}
  >
    <span class="icon" aria-hidden="true">{@html button.icon}</span>
  </button>
{/snippet}

<header class="toolbar">

  <ServerInfo/>

  <!-- Search -->
  {#each buttons as button}
    {#if button.pane === Pane.search}
      {@render toggle(button)}
    {/if}
  {/each}

  <div class="spacer"></div>

  <!-- Other Panes -->
  {#each buttons as button}
    {#if button.pane !== Pane.search}
      {@render toggle(button)}
    {/if}
  {/each}

  <div class="toolbar-user">
    <Identity/>
  </div>

</header>
