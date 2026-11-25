<script lang="ts">
import { MSG_LOAD_FAILED, MSG_LOADING } from "@/lib/labels";

interface Props {
    filename: string;
    media: boolean;
    failed: boolean;
}

let { filename, media, failed }: Props = $props();
</script>

<style>

    .media {
        display: flex;
        flex-direction: column;
        justify-content: center;
        align-items: center;
        text-align: center;
        height: var(--media-height);
        aspect-ratio: 4 / 3;
        max-width: 100%;
        padding: var(--control-padding);
        overflow: hidden;
        border: 1px solid var(--border);
    }

    .file {
        display: flex;
        flex-direction: column;
        justify-content: center;
        align-items: center;
        text-align: center;
        height: var(--file-height);
        width: var(--file-width);
        max-width: 100%;
        padding: 0 var(--control-padding);
        border: 1px solid var(--border);
    }

    .name {
        display: block;
        min-width: 0;
        max-width: 100%;
        overflow: hidden;
        white-space: nowrap;
        text-overflow: ellipsis;
    }

    .status {
        display: block;
        color: var(--text-dim);
        font-size: 0.85rem;
    }

</style>

<div class={media ? "media" : "file"}>
    <span class="name">{filename}</span>
    <span class="status">{failed ? MSG_LOAD_FAILED : MSG_LOADING}</span>
</div>
