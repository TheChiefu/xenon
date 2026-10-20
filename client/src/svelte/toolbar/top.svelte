<script lang="ts">
  import Identity from "./identity.svelte";
  import ServerInfo from "./server_info.svelte";
  import { Pane } from "@/lib/display/panes.svelte";
  import type { ToolbarButton } from "@/lib/display/toolbar";
  import { LocalEvent } from "@/lib/events";
  import * as events from "@/lib/events";
  import icon_search from "@/assets/icons/search.svg?raw";

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
    position: relative;
  }

  .pane-btn > .icon {
    opacity: 0.4;
  }

  .pane-btn.open > .icon {
    opacity: 1;
  }

  .pane-btn > .badge {
    position: absolute;
    top: 0;
    right: 0;
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
    {#if button.badge !== undefined && button.badge > 0}
      <span class="badge" aria-hidden="true">{button.badge}</span>
    {/if}
  </button>
{/snippet}

<header class="toolbar">

  <ServerInfo/>

  <div class="spacer"></div>

  <!-- Panes -->
  {#each buttons as button}
    {@render toggle(button)}
  {/each}

  <!-- Search -->
  <button class="icon-btn" aria-label="Search messages" title="Search messages" onclick={() => events.dispatch(LocalEvent.search_requested)}>
    <span class="icon" aria-hidden="true">{@html icon_search}</span>
  </button>

  <div class="toolbar-user">
    <Identity/>
  </div>

</header>
