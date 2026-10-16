<script lang="ts">
  import type { Snippet } from "svelte";
  import { Pane } from "@/lib/panes.svelte";
  import * as panes from "@/lib/panes.svelte";
  import { isCompact } from "@/lib/utils";

  interface Props {
    pane: Pane;
    overlay?: boolean;
    children: Snippet;
  }

  let { pane, overlay = false, children }: Props = $props();

  //UNDOCUMENTED
  function pressAway(event: PointerEvent): void {
    if (!isCompact.current) return;
    if (!(event.target instanceof Element)) return;
    if (event.target.closest(".pane, .bottom-toolbar, .menu, .menu-backdrop, dialog") !== null) return;

    panes.setOpenState(pane, false);
  }
</script>

<svelte:window onpointerdown={pressAway}/>

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


</style>

<div class="pane" class:overlay style:display={panes.isOpen(pane) ? null : "none"} style:--pane-width="{panes.getWidth(pane)}rem">
  {@render children()}
</div>
