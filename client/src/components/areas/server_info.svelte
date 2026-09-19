<script lang="ts">
import type {ServerInfo} from "@/bindings/routes/server";
import { getBaseUrl } from "@/lib/server";

let info = $state<ServerInfo | null>(null);
let error = $state<string | null>(null);

$effect(() => {
    fetch(`${getBaseUrl()}/server`)
        .then((response) => response.json())
        .then((data: ServerInfo) => { info = data; })
        .catch((err) => { error = err.message; });
});

</script>

<style>
    .server-title {
        font-size: 2em;
        font-style: italic;
        font-weight: bold;
    }

    .server-version {
        font-size: 1em;
        font-style: normal;
        font-weight: 100;
    }

    .server-kind {
        font-size: 0.65em;
        padding: 1px 6px;
        border-radius: 25%;
        background: #2f3643;
        color: #98a2b3;
        letter-spacing: 0.05em;
        text-transform: uppercase;
    }

    .server-description {
        font-size: 0.75em;
    }

</style>

<div class="server-info" title={info?.name}>
    {#if error}
        <span class="server-error">Could not reach server: {error}</span>
    {:else}
        <span class="server-title">{info?.name}</span>
        <span class="server-version">{info?.version}</span>
        <span class="server-kind">{info?.kind}</span>
        <span class="server-description">{info?.description}</span>
    {/if}
</div>