<script lang="ts">
import { onDestroy } from "svelte";
import type { AttachmentInfo } from "@/bindings/types/messages";
import type { ActionReturn } from "svelte/action";
import { fetchFileBlob, type FetchedFile } from "@/lib/utils";
import Image from "./attachments/image.svelte";
import Video from "./attachments/video.svelte";
import Audio from "./attachments/audio.svelte";
import File from "./attachments/file.svelte";
import Pending from "./attachments/pending.svelte";

interface Props {
    attachment: AttachmentInfo;
}

let { attachment }: Props = $props();
let revealed = $state(false);
let file: FetchedFile | null = $state(null);
let failed = $state(false);
const size = $derived(attachment.byte_size);
const isMedia = $derived(
    attachment.mime.startsWith("image/") ||
    attachment.mime.startsWith("video/")
);

let removed = false;

// Fetches the file once the attachment is on screen
function load(): void {
    fetchFileBlob(attachment.id, attachment.spoiler).then((fetched) => {

        // Revoke a URL fetched after the attachment was removed
        if (removed) {
            URL.revokeObjectURL(fetched.url);
            return;
        }

        file = fetched;
    }).catch(() => {
        failed = true;
    });
}

// Fetch file once visible
function lazyLoad(node: HTMLElement): ActionReturn {
    if (attachment.spoiler) {
        return {};
    }

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

// Reveals a spoiler and fetches its file
function reveal(): void {
    revealed = true;
    load();
}

// Revokes the file's URL once the attachment is removed
onDestroy(() => {
    removed = true;

    if (file !== null) {
        URL.revokeObjectURL(file.url);
    }
});

</script>

<div class="attachment" use:lazyLoad>

    {#if attachment.spoiler && !revealed}
        <button onclick={reveal}>Spoiler ({attachment.mime} - {size})</button>
    {:else if file === null}
        <Pending filename={attachment.filename} media={isMedia} {failed}/>
    {:else}
        {#if attachment.mime.startsWith("image/")}
            <Image url={file.url} filename={attachment.filename}/>
        {:else if attachment.mime.startsWith("video/")}
            <Video url={file.url} filename={attachment.filename}/>
        {:else if attachment.mime.startsWith("audio/")}
            <Audio url={file.url} filename={attachment.filename}/>
        {:else}
            <File url={file.url} filename={attachment.filename}/>
        {/if}
    {/if}

</div>
