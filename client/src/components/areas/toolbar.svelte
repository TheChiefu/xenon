<script lang="ts">
  import type { ServerInfo } from "@/bindings/routes/server";
  import { getUrl, getActiveLogin } from "@/lib/session.svelte";
  import { getServerInfo } from "@/lib/api/server";
  import Settings from "@/views/settings.svelte";
  import icon_refresh from "@/assets/icons/refresh.svg?raw";
  import icon_gear from "@/assets/icons/gear.svg?raw";
  import icon_headphones from "@/assets/icons/headphones.svg?raw";
  import icon_mic from "@/assets/icons/mic.svg?raw";
  import icon_search from "@/assets/icons/search.svg?raw";

  let info = $state<ServerInfo | null>(null);
  let settingsOpen = $state(false);

  $effect(() => {
    const url = getUrl();
    if (url === null) return;

    getServerInfo(url)
      .then((data) => { info = data; })
      .catch(() => { info = null; });
  });
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

  .server-title {
    display: flex;
    flex-wrap: wrap;
    align-content: center;
    align-items: center;
    gap: 0.0625rem 0.375rem;
    min-width: 0;
  }

  .server-title h1 {
    flex: 0 0 100%;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-size: 1rem;
    font-weight: 600;
  }

  .server-kind {
    order: 1;
    padding: 1px 6px;
    border: 1px solid var(--border);
    color: var(--text-dim);
    font-size: 0.64rem;
    letter-spacing: 0.04em;
    text-transform: uppercase;
  }

  .server-version {
    color: var(--text-dim);
    font-size: 0.72rem;
    white-space: nowrap;
  }

  .toolbar-tools {
    display: flex;
    align-items: center;
    gap: 0.25rem;
    margin-left: auto;
  }

  .user-chip {
    display: flex;
    align-items: center;
    gap: 0.625rem;
    height: 100%;
    padding: 0 1.125rem;
    border: 0;
    border-left: 1px solid var(--border);
    background: var(--component);
    color: var(--text);
    font: inherit;
    text-align: left;
    cursor: pointer;
  }

  .user-chip-name {
    font-weight: 600;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }
</style>

<header class="toolbar">

  <div class="toolbar-search">
    <input type="search" placeholder="Search" aria-label="Search"/>
    <button class="search-icon icon-btn" aria-hidden="true">
      <span class="icon">{@html icon_search}</span>
    </button>
  </div>

  <div class="server-title">
    <h1>{info?.name ?? ''}</h1>
    {#if info?.kind}
      <span class="server-kind">{info.kind}</span>
    {/if}
    {#if info?.version}
      <span class="server-version">v{info.version}</span>
    {/if}
  </div>

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

  {#if getActiveLogin() !== null}
    <button class="user-chip" aria-label="Your profile">
      <span class="user-chip-name">{getActiveLogin()?.username}</span>
    </button>
  {/if}

  {#if settingsOpen}
    <Settings onClose={() => settingsOpen = false}/>
  {/if}

</header>
