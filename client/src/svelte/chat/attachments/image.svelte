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

    .media {
        display: flex;
        gap: var(--inline-gap);
        max-width: 100%;
    }

    .open {
        display: block;
        min-width: 0;
        padding: 0;
        cursor: pointer;
    }

    .open:hover {
        background: none;
    }

    .entry {
        display: block;
        height: var(--media-height);
        width: auto;
        max-width: 100%;
        object-fit: contain;
    }

    .icon-column {
        display: flex;
        flex-direction: column;
        gap: var(--row-gap);
    }

</style>

<div class="media">
    <div class="icon-column">
        <a
            href={url}
            download={filename}
            aria-label="Download attachment"
            title="Download Attachment"
        >
            <span class="icon" aria-hidden="true">{@html download}</span>
        </a>
    </div>

    <button
        class="inherit open"
        aria-label="Open image"
        title="Open Image"
        onclick={() => events.dispatch(LocalEvent.image_selected, { url, filename })}
    >
        <img class="entry" src={url} alt={filename} />
    </button>
</div>
