<script lang="ts">
  import Identity from "@/components/identity.svelte";
  import ServerInfo from "@/components/server_info.svelte";

  import icon_search from "@/assets/icons/search.svg?raw";
</script>

<style>
  .toolbar {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    height: 4rem;
    flex: none;
    padding: 0 0 0 0.75rem;
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

  <button class="icon-btn" aria-label="Search">
    <span class="icon" aria-hidden="true">{@html icon_search}</span>
  </button>

  <div class="toolbar-user">
    <Identity/>
  </div>

</header>
