<script lang="ts">
  import type { Snippet } from "svelte";
  import { Pane } from "@/lib/display/panes.svelte";
  import * as panes from "@/lib/display/panes.svelte";

  interface Props {
    pane: Pane;
    children: Snippet;
  }

  let { pane, children }: Props = $props();
</script>

<style>

  

  .pane {
    display: flex;
    flex-direction: column;
    border: 1px solid var(--border);
    background: linear-gradient(
      var(--component-gradient-top, var(--component)),
      var(--component-gradient-bottom, var(--component))
    );
    overflow: hidden;
  }


</style>

{#if panes.isOpen(pane)}
  <div class="pane-backdrop" role="presentation" onpointerdown={() => panes.setOpenState(pane, false)}></div>
{/if}

<div class="pane pane-{pane}" style:display={panes.isOpen(pane) ? null : "none"}>
  {@render children()}
</div>
