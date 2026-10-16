<script lang="ts">
  import type { ToolbarButton } from "@/lib/toolbar";

  interface Props {
    buttons: ToolbarButton[];
  }

  let { buttons }: Props = $props();
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

  .pane-btn {
    opacity: 0.4;
  }

  .pane-btn.open {
    opacity: 1;
  }
</style>

<nav class="bottom-toolbar">
  {#each buttons as button}
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
