<script lang="ts">
  import * as server from "@/lib/connection/server.svelte";
  import * as socket from "@/lib/connection/socket.svelte";

  const info = $derived(server.getInfo());
  const name: string = $derived(info?.name ?? "");
  const connected: boolean = $derived(socket.isConnected());
</script>

<style>
  .frame {
    height: 100%;
    filter: drop-shadow(var(--status-bar-width) 0 0 var(--status-color));
  }

  .status-online    { --status-color: var(--status-online); }
  .status-offline   { --status-color: var(--status-offline); }

  .card {
    --point: 35px;

    height: 100%;
    align-content: center;
    align-items: center;
    background: var(--border);
    clip-path: polygon(
      0 0,                            /* Top Left */
      100% 0,                         /* Top Right */
      calc(100% - var(--point)) 50%,  /* Notch */
      100% 100%,                      /* Bottom Right */
      0 100%                          /* Bottom Left */
    );
    display: grid;
    gap: 0.2rem;
    grid-template-columns: auto 1fr;
    justify-items: start;
    padding: 0 var(--point) 0 0.75rem;
  }

  .title {
    font-size: 1rem;
    font-style: italic;
    font-weight: 600;
    grid-column: 1 / -1;
    max-width: 100%;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    user-select: none;
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
    user-select: none;
  }

  .version {
    color: var(--text-dim);
    font-size: 0.72rem;
    grid-column: 1;
    grid-row: 2;
    white-space: nowrap;
    user-select: none;
  }
</style>

<div class="server-info" title={connected ? "Connected" : "Disconnected"}>
  <div class="frame {connected ? "status-online" : "status-offline"}">
    <div class="card">
      <h1 class="title" title={name}>{name}</h1>
      {#if info?.kind}
        <span class="kind">{info.kind}</span>
      {/if}
      {#if info?.version}
        <span class="version">v{info.version}</span>
      {/if}
    </div>
  </div>
</div>
