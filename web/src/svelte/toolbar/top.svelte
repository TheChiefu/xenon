<script lang="ts">
  import Identity from "./identity.svelte";
  import ServerInfo from "./server_info.svelte";
  import type { ToolbarButton } from "@/lib/display/toolbar";
  import { Kind } from "@/lib/display/toolbar";

  interface Props {
    buttons: ToolbarButton[];
  }

  let { buttons }: Props = $props();
  let regularButtons = $derived(buttons.filter((b) => b.kind === Kind.Regular));
  let overlayButtons = $derived(buttons.filter((b) => b.kind === Kind.Overlay));
  let modalButtons = $derived(buttons.filter((b) => b.kind === Kind.Modal));

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

  .divider-vertical {
    height: 1.5rem;
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

  <!-- Regular Panes -->
  {#each regularButtons as button}
    {@render toggle(button)}
  {/each}

  <div class="spacer"></div>

  <!-- Overlay Panes -->
  {#each overlayButtons as button}
    {@render toggle(button)}
  {/each}

  {#if buttons.length > 0}
    <div class="divider-vertical"></div>
  {/if}

  <!-- Modal Panes -->
  {#each modalButtons as button}
    {@render toggle(button)}
  {/each}

  <div class="toolbar-user">
    <Identity/>
  </div>

</header>
