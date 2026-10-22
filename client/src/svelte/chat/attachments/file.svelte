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
        border: 1px solid var(--border);
    }

    .name {
        min-width: 0;
        overflow: hidden;
        white-space: nowrap;
        text-overflow: ellipsis;
        user-select: none;
    }

</style>

<a
    class="file"
    href={url}
    download={filename}
    aria-label="Download attachment"
    title="Download Attachment"
>
    <span class="icon" aria-hidden="true">{@html download}</span>
    <span class="name">{filename}</span>
</a>
