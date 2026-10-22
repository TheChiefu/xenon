<script lang="ts">
  import Members from "./members.svelte";
  import Call from "./call.svelte";
  import { DetailsTab } from "@/lib/display/panes.svelte";

  interface Props {
    roomId: string | null;
    open: boolean;
  }

  let { roomId, open }: Props = $props();

  let current_tab: DetailsTab = $state(DetailsTab.members);
</script>

<style>

  .tabs {
    display: flex;
    flex: none;
  }

  .tabs > button {
    flex: 1;
  }

</style>

<div class="tabs">
  <button
    class="is-tab"
    class:is-active={current_tab === DetailsTab.members}
    onclick={() => current_tab = DetailsTab.members}
  >
    Members
  </button>
  <button
    class="is-tab"
    class:is-active={current_tab === DetailsTab.call}
    onclick={() => current_tab = DetailsTab.call}
  >
    Call
  </button>
</div>

{#if current_tab === DetailsTab.members}
  <Members {roomId} {open}/>
{:else}
  <Call {roomId}/>
{/if}
