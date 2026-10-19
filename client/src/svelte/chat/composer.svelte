<script lang="ts">
  import type { AttachmentRequest } from "@/bindings/shared";
  import * as filesAPI from "@/lib/api/files";
  import * as messagesAPI from "@/lib/api/messages";
  import * as messages from "@/lib/messages/data.svelte";
  import * as settings from "@/lib/client/settings.svelte";
  import { SendKey } from "@/lib/client/settings.svelte";
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

  // Message text area, resized to fit its text
  let text_input: HTMLTextAreaElement | undefined = $state();

  // Resizes the text area to its content height whenever the text changes
  $effect(() => {
    text;
    if (text_input === undefined) {
      return;
    }

    // Reset first so the text area can also shrink, then add the border to the content height
    text_input.style.height = "auto";
    text_input.style.height = `${text_input.scrollHeight + text_input.offsetHeight - text_input.clientHeight}px`;
  });

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

  // EVENTS //

  // Determine how key events are handled
  function onKeydown(event: KeyboardEvent): void {

    // Only send on allowed modifier combinations
    if (isSendKey(event)) {
      event.preventDefault();
      send();
    }
  }

  // Handle drag and drop of files into text area
  function onDrop(event: DragEvent) {

    const files = event.dataTransfer?.files;

    // Nothing to parse
    if (!files || files.length === 0) { return; }

    // Read all dropped files
    event.preventDefault();
    addFiles(files);
  }

  // Handle pasting events into text area
  function onPaste(event: ClipboardEvent) {

    const files = event.clipboardData?.files;

    // Just text, passthrough as normal
    if (!files || files.length === 0) { return; }

    // Rename files to append "clipboard" to them and by random UUID
    const renamed: File[] = [];
    for (const file of files) {
      renamed.push(filesAPI.generateName(file, "clipboard"));
    }

    // Parse files
    event.preventDefault();
    addFiles(renamed);
  }

  // METHODS //

  function isSendKey(event: KeyboardEvent): boolean {
    if (event.key !== "Enter" || event.isComposing) {
      return false;
    }

    switch (settings.getSendKey()) {
      case SendKey.Enter:
        return !event.ctrlKey && !event.shiftKey;
      case SendKey.CtrlEnter:
        return event.ctrlKey;
      case SendKey.ShiftEnter:
        return event.shiftKey;
    }
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

  function onPickFiles(): void {
    if (file_input === undefined || file_input.files === null) {
      return;
    }

    addFiles(file_input.files);

    // Clear the input so the same file can be selected again
    file_input.value = "";
  }

  // Adds selected local files as attachments
  function addFiles(files: Iterable<File>): void {
    const limit = messagesAPI.attachmentLimit();

    for (const file of files) {

      // Stop at the server's attachment limit
      if (limit !== null && attachments.length >= limit) {
        failure = `A message can have at most ${limit} attachments`;
        break;
      }

      attachments.push({ file: file, id: null, spoiler: false });
    }
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
    box-sizing: border-box;
    max-height: 20rem;
    overflow-y: auto;
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

    <textarea
      placeholder="Message"
      rows="2"
      bind:this={text_input}
      bind:value={text}
      onpaste={onPaste}
      onkeydown={onKeydown}
      ondragover={(event) => event.preventDefault()}
      ondrop={onDrop}
    ></textarea>

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

  <!-- Hidden file input picker uses to bind to -->
  <input type="file" multiple bind:this={file_input} onchange={onPickFiles} hidden/>
</div>
