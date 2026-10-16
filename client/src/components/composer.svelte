<script lang="ts">
  import * as messagesAPI from "@/lib/api/messages";
  import * as messages from "@/lib/messages.svelte";
  import { failureText } from "@/lib/utils";

  interface Props {
    roomId: string;
  }

  let { roomId }: Props = $props();

  let text: string = $state("");
  let failure: string | null = $state(null);
  let sending = false;

  // Kept after a failed send, so a retry cannot store the message twice
  let nonce: string = newNonce();

  // Creates a new nonce when the room changes
  $effect(() => {
    roomId;
    nonce = newNonce();
  });

  // Ctrl+Enter sends, Enter alone adds a new line
  function onKeydown(event: KeyboardEvent): void {
    if (event.key !== "Enter" || !event.ctrlKey) {
      return;
    }

    event.preventDefault();
    send();
  }

  // Posts the text, adding the stored message the server returns
  async function send(): Promise<void> {
    if (sending || text.trim() === "") {
      return;
    }

    sending = true;

    try {
      const message = await messagesAPI.post(roomId, { body: text, client_nonce: nonce, attachments: [] });
      messages.add(message);

      text = "";
      failure = null;
      nonce = newNonce();
    } catch (error) {
      failure = failureText(error);
    } finally {
      sending = false;
    }
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
    flex: 1;
    gap: 0.25rem;
  }

  textarea {
    resize: none;
  }
</style>

<div class="composer-box">
  {#if failure !== null}
    <p>{failure}</p>
  {/if}

  <textarea placeholder="Message" rows="2" bind:value={text} onkeydown={onKeydown}></textarea>
</div>
