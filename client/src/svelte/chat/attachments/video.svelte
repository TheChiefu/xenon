<script lang="ts">
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

    .entry {
        height: var(--media-height);
        min-width: 0;
        max-width: 100%;
        padding: 1rem;
        border: 1px solid var(--border);
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

    <video class="entry" src={url} controls></video>
</div>
