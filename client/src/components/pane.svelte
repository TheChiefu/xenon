<script lang="ts">
  import type { Snippet } from "svelte";
  import { Pane, getWidth } from "@/lib/panes.svelte";

  interface Props {
    pane: Pane;
    overlay?: boolean;
    children: Snippet;
  }

  let { pane, overlay = false, children }: Props = $props();
</script>

<style>

  .pane {
    position: relative;
    display: flex;
    flex-direction: column;
    flex: none;
    width: var(--pane-width);
    border: 1px solid var(--border);
    background: var(--component);
    overflow: hidden;
  }

  .pane.overlay {
    position: absolute;
    top: 0;
    right: 0;
    bottom: 0;
  }

  @media (width < 1000px) {
    .pane {
      position: absolute;
      top: 0;
      right: auto;
      bottom: 0;
      left: 0;
      width: 75%;
    }
  }

</style>

<div class="pane" class:overlay style:--pane-width="{getWidth(pane)}rem">
  {@render children()}
</div>
