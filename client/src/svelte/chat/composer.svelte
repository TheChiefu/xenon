<script lang="ts">
  import type { AttachmentRequest } from "@/bindings/shared";
  import * as filesAPI from "@/lib/api/files";
  import * as messagesAPI from "@/lib/api/messages";
  import * as messages from "@/lib/messages/data.svelte";
  import { failureText } from "@/lib/utils";
  import DraftAttachmentRow, { type DraftAttachment } from "./draft_attachment.svelte";

  import icon_clip from "@/assets/icons/clip.svg?raw";
  import icon_send from "@/assets/icons/send.svg?raw";

  interface Props {
    roomId: string;
  }

  let { roomId }: Props = $props();

  let text: string = $state("");
  let failure: string | null = $state(null);
  let sending: boolean = $state(false);

  // File input for selecting local files
  let file_input: HTMLInputElement | undefined = $state();

  // Local files waiting for upload on send
  let attachments: DraftAttachment[] = $state([]);

  // Kept after a failed send, so a retry cannot store the message twice
  let nonce: string = newNonce();

  // Creates a new nonce and clears attachments when the room changes
  $effect(() => {
    roomId;
    nonce = newNonce();
    attachments = [];
  });

  // Ctrl+Enter sends, Enter alone adds a new line
  function onKeydown(event: KeyboardEvent): void {
    if (event.key !== "Enter" || !event.ctrlKey) {
      return;
    }

    event.preventDefault();
    send();
  }

  // Posts the text and attachments
  async function send(): Promise<void> {
    if (!canSend()) {
      return;
    }

    sending = true;

    try {
      const requests: AttachmentRequest[] = [];
      for (const attachment of attachments) {

        // Upload a file that has not been uploaded yet
        if (attachment.id === null) {
          const stored = await filesAPI.upload(attachment.file);
          attachment.id = stored.id;
        }

        requests.push({ file_id: attachment.id, spoiler: attachment.spoiler });
      }

      // Send no body for a message with only attachments
      let body: string | null = null;
      if (text.trim() !== "") {
        body = text;
      }

      const message = await messagesAPI.post(roomId, { body: body, client_nonce: nonce, attachments: requests });
      messages.add(message);

      // Clear the composer once sent
      text = "";
      attachments = [];
      failure = null;
      nonce = newNonce();
    } catch (error) {
      failure = failureText(error);
    } finally {
      sending = false;
    }
  }

  // Whether a message can be sent
  function canSend(): boolean {

    // Not during a send
    if (sending) {
      return false;
    }

    // Only with text or an attachment
    return text.trim() !== "" || attachments.length > 0;
  }

  // Adds selected local files as attachments
  function addFiles(): void {
    if (file_input === undefined || file_input.files === null) {
      return;
    }

    const limit = messagesAPI.attachmentLimit();

    for (const file of file_input.files) {

      // Stop at the server's attachment limit
      if (limit !== null && attachments.length >= limit) {
        failure = `A message can have at most ${limit} attachments`;
        break;
      }

      attachments.push({ file: file, id: null, spoiler: false });
    }

    // Clear the input so the same file can be selected again
    file_input.value = "";
  }

  // Removes an attachment
  // - attachment: Attachment to remove
  function removeAttachment(attachment: DraftAttachment): void {
    attachments = attachments.filter((other) => other !== attachment);
  }

  // 16 random bytes as hex
  function newNonce(): string {
    const bytes = crypto.getRandomValues(new Uint8Array(16));

    let hex = "";
    for (const byte of bytes) {
      hex += byte.toString(16).padStart(2, "0");
    }

    return hex;
  }
</script>

<style>
  .composer-box {
    display: flex;
    flex-direction: column;
    gap: 0.25rem;
  }

  .composer-row {
    display: flex;
    align-items: center;
    gap: var(--inline-gap);
  }

  .attachments {
    display: flex;
    flex-wrap: wrap;
    gap: var(--inline-gap);
  }

  textarea {
    flex: 1;
    resize: none;
  }

  .send-btn {
    align-self: center;
    width: 3rem;
    height: 3rem;
    border-radius: 0;
  }
</style>

<div class="composer-box">
  {#if failure !== null}
    <p>{failure}</p>
  {/if}

  {#if attachments.length > 0}
    <div class="attachments">
      {#each attachments as attachment, index}
        <DraftAttachmentRow
          bind:attachment={attachments[index]}
          removeCallback={() => removeAttachment(attachment)}
        />
      {/each}
    </div>
  {/if}

  <div class="composer-row">
    <button
      class="icon-btn"
      aria-label="Attach files"
      title="Attach Files"
      onclick={() => file_input?.click()}
      disabled={sending}
    >
      <span class="icon" aria-hidden="true">{@html icon_clip}</span>
    </button>

    <textarea placeholder="Message" rows="2" bind:value={text} onkeydown={onKeydown}></textarea>

    <button
      class="icon-btn is-action send-btn"
      aria-label="Send message"
      title="Send Message"
      onclick={send}
      disabled={!canSend()}
    >
      <span class="icon" aria-hidden="true">{@html icon_send}</span>
    </button>
  </div>

  <input type="file" multiple bind:this={file_input} onchange={addFiles} hidden/>
</div>
