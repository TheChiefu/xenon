<script lang="ts">
  import { GlobalRole } from "@/bindings/shared";
  import ProfileCard from "@/components/profile_card.svelte";
  import icon_x from "@/assets/icons/x.svg?raw";

  interface Props {
    onClose: () => void;
  }
  let { onClose }: Props = $props();
  let dialog = $state<HTMLDialogElement | null>(null);

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

</style>

<dialog bind:this={dialog} onclose={onClose}>

  <div class="header">
    <button class="icon-btn" aria-label="Close" onclick={() => dialog?.close()}>
      <span class="icon" aria-hidden="true">{@html icon_x}</span>
    </button>
  </div>

  <!-- Placeholder values until the profile is wired in -->
  <ProfileCard
    id="00000000000000000000000000000000"
    display_name="Display Name"
    username="username"
    description=""
    role={GlobalRole.member}
    links={[]}
    created_at="Jan 1, 2026"
  />

</dialog>
