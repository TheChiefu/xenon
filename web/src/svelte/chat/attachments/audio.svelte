<script lang="ts">
import download from "@/assets/icons/download.svg?raw";

interface Props {
    url: string;
    filename: string;
}

let { url, filename }: Props = $props();
</script>

<style>

    .file {
        display: flex;
        align-items: center;
        gap: var(--inline-gap);
        height: var(--file-height);
        width: var(--file-width);
        max-width: 100%;
        padding: 0 var(--control-padding);
    }

    .entry {
        flex: 1;
        min-width: 0;
    }

</style>

<div class="file">
    <a
        href={url}
        download={filename}
        aria-label="Download attachment"
        title="Download Attachment"
    >
        <span class="icon" aria-hidden="true">{@html download}</span>
    </a>

    <audio class="entry" src={url} controls></audio>
</div>
