<script lang="ts">
import type {ServerInfo} from "@/bindings/routes/server";
import { getUrl } from "@/lib/session.svelte";
import { getServerInfo } from "@/lib/api/server";

let info = $state<ServerInfo | null>(null);
let error = $state<string | null>(null);

$effect(() => {
    const url = getUrl();
    if (url === null) {
        return;
    }

    getServerInfo(url)
        .then((data) => { info = data; })
        .catch((err) => { error = err.message; });
});

</script>

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
