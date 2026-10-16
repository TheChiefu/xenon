<script lang="ts">
  import Identity from "@/components/identity.svelte";
  import ServerInfo from "@/components/server_info.svelte";
  import PaneButtons from "@/components/toolbar/pane_buttons.svelte";
  import { isCompact } from "@/lib/utils";
</script>

<style>
  .toolbar {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    height: 4rem;
    flex: none;
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

</style>

<header class="toolbar">


  <ServerInfo/>

  <div class="spacer"></div>

  {#if !isCompact.current}
    <PaneButtons/>
  {/if}

  <div class="toolbar-user">
    <Identity/>
  </div>

</header>
