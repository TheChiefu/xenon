<script lang="ts">
interface Props {
    filename: string;
    media: boolean;
}

let { filename, media }: Props = $props();
</script>

<style>

    .media {
        height: var(--media-height);
        aspect-ratio: 4 / 3;
        max-width: 100%;
        padding: var(--control-padding);
        overflow: hidden;
        border: 1px solid var(--border);
    }

    .file {
        display: flex;
        align-items: center;
        height: var(--file-height);
        width: var(--file-width);
        max-width: 100%;
        padding: 0 var(--control-padding);
        border: 1px solid var(--border);
    }

    .name {
        display: block;
        min-width: 0;
        overflow: hidden;
        white-space: nowrap;
        text-overflow: ellipsis;
    }

</style>

<div class={media ? "media" : "file"}>
    <span class="name">{filename}</span>
</div>
