<script lang="ts">
import download from "@/assets/icons/download.svg?raw";

interface Props {
    url: string;
    filename: string;
}

let { url, filename }: Props = $props();
</script>

<style>

    .entry {
        padding: 1rem;
        max-width: 100%;
        max-height: 40rem;
        border: 1px solid var(--border);
    }

    .icon-row {
        display: flex;
        gap: var(--inline-gap);
    }

</style>

<video class="entry" src={url} controls></video>

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
