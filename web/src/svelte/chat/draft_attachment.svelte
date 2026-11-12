<script module lang="ts">
  export interface DraftAttachment {
    file: File;
    id: string | null;
    spoiler: boolean;
  }
</script>

<script lang="ts">
  import icon_eye from "@/assets/icons/eye.svg?raw";
  import icon_eye_off from "@/assets/icons/eye-off.svg?raw";
  import icon_x from "@/assets/icons/x.svg?raw";

  interface Props {
    attachment: DraftAttachment;
    removeCallback: () => void;
  }

  let { attachment = $bindable(), removeCallback }: Props = $props();
</script>

<style>
  .draft-attachment {
    display: flex;
    align-items: center;
    gap: var(--inline-gap);
    padding-left: var(--control-padding);
    border: 1px solid var(--border);
  }
</style>

<div class="draft-attachment">
  <span>{attachment.file.name}</span>

  <button
    class="icon-btn"
    aria-label="Spoiler"
    title="Spoiler"
    onclick={() => attachment.spoiler = !attachment.spoiler}
  >
    <span class="icon" aria-hidden="true">{@html attachment.spoiler ? icon_eye_off : icon_eye}</span>
  </button>

  <button
    class="icon-btn"
    aria-label="Remove attachment"
    title="Remove Attachment"
    onclick={removeCallback}
  >
    <span class="icon" aria-hidden="true">{@html icon_x}</span>
  </button>
</div>
