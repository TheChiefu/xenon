<script lang="ts">
  import Identity from "@/components/identity.svelte";
  import ServerInfo from "@/components/server_info.svelte";
  import Settings from "@/views/settings.svelte";

  import icon_refresh from "@/assets/icons/refresh.svg?raw";
  import icon_gear from "@/assets/icons/gear.svg?raw";
  import icon_headphones from "@/assets/icons/headphones.svg?raw";
  import icon_mic from "@/assets/icons/mic.svg?raw";
  import icon_search from "@/assets/icons/search.svg?raw";

  let settingsOpen = $state(false);
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

  .toolbar-search {
    flex: 0 1 16rem;
    position: relative;
  }

  .toolbar-search input {
    height: 2.1rem;
    padding: 0 2rem 0 0.625rem;
  }

  .search-icon {
    position: absolute;
    top: 50%;
    right: 0.5rem;
    transform: translateY(-50%);
    pointer-events: none;
    color: var(--text-dim);
  }

  .toolbar-tools {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    margin-left: auto;
  }

</style>

<header class="toolbar">

  <div class="toolbar-search">
    <input type="search" placeholder="Search" aria-label="Search"/>
    <button class="search-icon icon-btn" aria-hidden="true">
      <span class="icon">{@html icon_search}</span>
    </button>
  </div>

  <ServerInfo/>

  <div class="toolbar-tools">
    <button class="icon-btn" aria-label="Check for updates">
      <span class="icon" aria-hidden="true">{@html icon_refresh}</span>
    </button>
    <button class="icon-btn" aria-label="Client settings" onclick={() => settingsOpen = true}>
      <span class="icon" aria-hidden="true">{@html icon_gear}</span>
    </button>
    <button class="icon-btn" aria-label="Toggle sound" disabled>
      <span class="icon" aria-hidden="true">{@html icon_headphones}</span>
    </button>
    <button class="icon-btn" aria-label="Toggle microphone" disabled>
      <span class="icon" aria-hidden="true">{@html icon_mic}</span>
    </button>
  </div>

  <Identity/>

  {#if settingsOpen}
    <Settings onClose={() => settingsOpen = false}/>
  {/if}

</header>
