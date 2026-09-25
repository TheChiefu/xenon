<script lang="ts">
  import ProfileCard from "@/components/profile_card.svelte";
  import icon_x from "@/assets/icons/x.svg?raw";

  interface Props {
    onClose: () => void;
  }
  let { onClose }: Props = $props();
  let dialog = $state<HTMLDialogElement | null>(null);

  // Unsaved values, drawn on the preview as they are typed
  let display_name = $state("");
  let description = $state("");

  $effect(() => {
    dialog?.showModal();
  });
</script>

<style>
  dialog {
    width: 44rem;
    padding: 1.5rem;
    border: 1px solid var(--border);
    background: var(--component);
    color: var(--text);
  }

  dialog::backdrop {
    background: var(--modal-background)
  }

  .header {
    display: flex;
    justify-content: space-between;
    margin-bottom: 1.0rem;
    border-bottom: 1px solid var(--border);
  }

  h2 {
    margin-bottom: 0.5rem;
    font-size: 1.25rem;
  }

  .editor {
    display: flex;
    gap: 1.5rem;
    align-items: flex-start;
  }

  .form {
    display: flex;
    flex-direction: column;
    flex: 1;
    gap: 0.5rem;
  }

  textarea {
    min-height: 10rem;
  }
</style>

<dialog bind:this={dialog} onclose={onClose}>

  <div class="header">
    <h2>Edit Profile</h2>
    <button class="icon-btn" aria-label="Close" onclick={() => dialog?.close()}>
      <span class="icon" aria-hidden="true">{@html icon_x}</span>
    </button>
  </div>

  <div class="editor">
    <div class="form">
      <label for="display-name">Display Name</label>
      <input id="display-name" placeholder="Name visible to others" bind:value={display_name}/>

      <label for="description">Description</label>
      <textarea id="description" bind:value={description}></textarea>

      <button>Save Changes</button>
    </div>

    <!-- Placeholder values until the profile is wired in -->
    <ProfileCard
      id="00000000000000000000000000000000"
      display_name={display_name}
      username="username"
      description={description}
      created_at="Jan 1, 2026"
      editable
    />
  </div>

</dialog>
