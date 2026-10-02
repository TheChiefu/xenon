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

  div {
    --point: 40px;

    align-content: center;
    align-items: center;
    align-self: stretch;
    background: var(--border);
    clip-path: polygon(
      0 0,                            /* Top Left */
      calc(100% - var(--point)) 0,    /* Top Edge */
      100% 50%,                       /* Point */
      calc(100% - var(--point)) 100%, /* Bottom Edge */
      0 100%                          /* Bottom Left */
    );
    display: grid;
    gap: 0.2rem;
    grid-template-columns: auto 1fr;
    justify-items: start;
    padding: 0 var(--point) 0 0.75rem;
    width: fit-content;
  }

  .title {
    font-size: 1rem;
    font-style: italic;
    font-weight: 600;
    grid-column: 1 / -1;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .kind {
    background-color: var(--component);
    border: 1px solid var(--border);
    border-radius: 1rem;
    color: var(--text-dim);
    font-size: 0.65rem;
    grid-column: 2;
    grid-row: 2;
    order: 1;
    padding: 1px 6px;
    text-transform: capitalize;
  }

  .version {
    color: var(--text-dim);
    font-size: 0.72rem;
    grid-column: 1;
    grid-row: 2;
    white-space: nowrap;
  }
</style>

<div>
  <h1 class="title">{info?.name ?? ''}</h1>
  {#if info?.kind}
    <span class="kind">{info.kind}</span>
  {/if}
  {#if info?.version}
    <span class="version">v{info.version}</span>
  {/if}
</div>
