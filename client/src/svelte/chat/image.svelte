<script lang="ts">
import { LocalEvent } from "@/lib/events";
import * as events from "@/lib/events";
import download from "@/assets/icons/download.svg?raw";

interface Props {
    url: string;
    filename: string;
}

let { url, filename }: Props = $props();
</script>

<style>

    .open {
        display: block;
        max-width: 100%;
        padding: 0;
        cursor: pointer;
    }

    .open:hover {
        background: none;
    }

    .entry {
        max-width: 100%;
        max-height: 40rem;
    }

    .icon-row {
        display: flex;
        gap: var(--inline-gap);
    }

</style>

<button
    class="inherit open"
    aria-label="Open image"
    title="Open Image"
    onclick={() => events.dispatch(LocalEvent.image_selected, { url, filename })}
>
    <img class="entry" src={url} alt={filename} />
</button>

<div class="icon-row">
    <a
        href={url}
        download={filename}
        aria-label="Download attachment"
        title="Download Attachment"
    >
        <span class="icon" aria-hidden="true">{@html download}</span>
    </a>
</div>
