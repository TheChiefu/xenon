<script lang="ts">
  import type { ServerInfo } from "@/bindings/routes/server";
  import { getUrl } from "@/lib/session.svelte";
  import { getServerInfo } from "@/lib/api/server";

  let info = $state<ServerInfo | null>(null);

  $effect(() => {
    const url = getUrl();
    if (url === null) return;

    getServerInfo(url)
      .then((data) => { info = data; })
      .catch(() => { info = null; });
  });
</script>

<style>
  .server-title {
    display: flex;
    flex-wrap: wrap;
    align-content: center;
    align-items: center;
    gap: 0.0625rem 0.375rem;
    min-width: 6rem;
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
</style>

<div class="server-title">
  <h1>{info?.name ?? ''}</h1>
  {#if info?.kind}
    <span class="server-kind">{info.kind}</span>
  {/if}
  {#if info?.version}
    <span class="server-version">v{info.version}</span>
  {/if}
</div>
