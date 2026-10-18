<script lang="ts">
import type { AttachmentResponse } from "@/bindings/routes/messages";
import { fetchFileBlob, type FetchedFile } from "@/lib/utils";
import Image from "./image.svelte";
import Video from "./video.svelte";
import download from "@/assets/icons/download.svg?raw";

interface Props {
    attachment: AttachmentResponse;
}

let { attachment }: Props = $props();
let revealed = $state(false);
let file: FetchedFile | null = $state(null);
const size = $derived(attachment.byte_size);

// Fetches the file, revoking its URL once the attachment is removed
$effect(() => {
    let removed = false;

    fetchFileBlob(attachment.id).then((fetched) => {

        // Revoke a URL fetched after the attachment was removed
        if (removed) {
            URL.revokeObjectURL(fetched.url);
            return;
        }

        file = fetched;
    });

    return () => {
        removed = true;

        if (file !== null) {
            URL.revokeObjectURL(file.url);
        }
    };
});

</script>

<style>

    .unknown {
        user-select: none;
    }

    .entry {
        padding: 1rem;
        max-width: 100%;
        max-height: 40rem;
    }

    .icon-row {
        display: flex;
        gap: var(--inline-gap);
    }

</style>

<div class="attachment">

    {#if attachment.spoiler && !revealed}
        <button onclick={() => revealed = true}>Spoiler ({attachment.mime} - {size})</button>
    {:else if file === null}
        <span>{attachment.filename}</span>
    {:else}
        {#if attachment.mime.startsWith("image/")}
            <Image url={file.url} filename={attachment.filename}/>
        {:else if attachment.mime.startsWith("video/")}
            <Video url={file.url} filename={attachment.filename}/>
        {:else if attachment.mime.startsWith("audio/")}
            <audio class="entry" src={file.url} controls></audio>
        {:else}
            <a
                href={file.url}
                download={attachment.filename}
                aria-label="Download attachment"
                title="Download Attachment"
            >
            <span class="entry unknown">[{attachment.filename}]</span>
            <span class="icon" aria-hidden="true">{@html download}</span>
            </a>
        {/if}

        <div class="icon-row">
        </div>
    {/if}

</div>
