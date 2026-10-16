<script lang="ts">
  import { dialogFly } from "@/lib/transitions";
  import Codes from "@/components/admin/codes.svelte";
  import Users from "@/components/admin/users.svelte";
  import icon_x from "@/assets/icons/x.svg?raw";

  interface Props {
    onClose: () => void;
  }

  let { onClose }: Props = $props();
  let dialog = $state<HTMLDialogElement | null>(null);
  let current_tab = $state(0);

  const tabs = [
    { label: "Invites", view: Codes },
    { label: "Users", view: Users },
  ];

  const View = $derived(tabs[current_tab].view);

  $effect(() => {
    dialog?.showModal();
  });

  function cancel(event: Event) {
    if (event.target === dialog) {
      event.preventDefault();
      onClose();
    }
  }

  function pressAway(event: PointerEvent) {
    if (event.target === dialog) {
      onClose();
    }
  }
</script>

<style>
  dialog {
    width: 50rem;
    padding: 1.5rem;
    border: 1px solid var(--border);
    background: var(--component);
    color: var(--text);
  }

  dialog::backdrop {
    background: var(--modal-background);
  }

  .header {
    display: flex;
    justify-content: space-between;
    margin-bottom: 1rem;
    border-bottom: 1px solid var(--border);
  }

  h2 {
    font-size: 1.25rem;
    margin-bottom: 0.5rem;
  }

  .body {
    display: grid;
    grid-template-columns: 10rem 1fr;
    height: 30rem;
  }

  .tabs {
    display: grid;
    align-content: start;
  }

  .tabs button {
    text-align: left;
  }

  .tab-view {
    padding: 1rem;
    overflow-y: auto;
    background-color: var(--background);
  }
</style>

<dialog bind:this={dialog} onclose={onClose} oncancel={cancel} onpointerdown={pressAway} transition:dialogFly={{ y: "1rem" }}>
  <div class="header">
    <h2>Admin</h2>
    <button class="icon-btn" aria-label="Close" onclick={onClose}>
      <span class="icon" aria-hidden="true">{@html icon_x}</span>
    </button>
  </div>

  <div class="body">
    <div class="tabs">
      {#each tabs as tab, index}
        <button
          class="is-tab"
          class:is-active={current_tab === index}
          onclick={() => current_tab = index}
        >
          {tab.label}
        </button>
      {/each}
    </div>

    <div class="tab-view">
      <View/>
    </div>
  </div>
</dialog>
