<script lang="ts">
import { onDestroy } from "svelte";
import type { AttachmentResponse } from "@/bindings/routes/messages";
import type { ActionReturn } from "svelte/action";
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

let removed = false;

// Fetches the file once the attachment is on screen
function load(): void {
    fetchFileBlob(attachment.id).then((fetched) => {

        // Revoke a URL fetched after the attachment was removed
        if (removed) {
            URL.revokeObjectURL(fetched.url);
            return;
        }

        file = fetched;
    });
}

// Fetch file once visible
function lazyLoad(node: HTMLElement): ActionReturn {
    const observer = new IntersectionObserver((entries) => {
        if (entries.some((entry) => entry.isIntersecting)) {
            observer.disconnect();
            load();
        }
    });

    observer.observe(node);

    return {
        destroy() {
            observer.disconnect();
        },
    };
}

// Revokes the file's URL once the attachment is removed
onDestroy(() => {
    removed = true;

    if (file !== null) {
        URL.revokeObjectURL(file.url);
    }
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

<div class="attachment" use:lazyLoad>

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
