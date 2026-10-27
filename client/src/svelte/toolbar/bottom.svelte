<script lang="ts">
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
  .bottom-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-around;
    height: 4rem;
    flex: none;
    border-top: 1px solid var(--border);
    background: var(--component);
  }

  .divider-vertical {
    height: 1.5rem;
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

<nav class="bottom-toolbar">

  <!-- Regular and Overlay Panes are always shown on left, on mobile -->
  {#each regularButtons as button}
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
  {/each}

  {#each overlayButtons as button}
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
  {/each}

  <div class="divider-vertical"></div>

  <!-- Modal Buttons on right -->
  {#each modalButtons as button}
    <button
      class="icon-btn pane-btn"
      class:open={button.active}
      aria-label={button.label}
      title={button.label}
      onclick={button.onclick}
    >
      <span class="icon" aria-hidden="true">{@html button.icon}</span>
    </button>
  {/each}

</nav>
