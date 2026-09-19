<script lang="ts">
import type { AttachmentResponse } from "@/bindings/routes/messages";
import download from "@/assets/icons/download.svg";

interface Props {
    attachment: AttachmentResponse;
}

let { attachment }: Props = $props();
let revealed = $state(false);
const size = $derived(attachment.byte_size);
const url = $derived(`/files/${attachment.id}`);

</script>

<div class="attachment">

    {#if attachment.spoiler && !revealed}
        <button onclick={() => revealed = true}>Spoiler ({attachment.mime} - {size})</button>
    {:else if attachment.mime.startsWith("image/")}
        <img src={url} alt={attachment.filename} />
    {:else if attachment.mime.startsWith("video/")}
        <video src={url} controls></video>
    {:else if attachment.mime.startsWith("audio/")}
        <audio src={url} controls></audio>
    {:else}
        <a href={url} download>{attachment.filename}<img src={download} alt="Download Button"/></a>
    {/if}

</div>