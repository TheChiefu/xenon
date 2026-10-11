<script lang="ts">
  import { dialogFly } from "@/lib/transitions";
  import Profile from "@/components/settings/profile.svelte";
  import Client from "@/components/settings/client.svelte";
  import Account from "@/components/settings/account.svelte";
  import icon_x from "@/assets/icons/x.svg?raw";

  interface Props {
    onClose: () => void;
  }

  let { onClose }: Props = $props();
  let dialog = $state<HTMLDialogElement | null>(null);
  let current_tab = $state(0);

  const tabs = [
    { label: "Profile", view: Profile },
    { label: "Client", view: Client },
    { label: "Account", view: Account },
  ];

  const Tabs = $derived(tabs[current_tab].view);

  $effect(() => {
    dialog?.showModal();
  });

  // Closes on Escape with the fade out
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
    background: var(--modal-background)
  }

  .header {
    display: flex;
    justify-content: space-between;
    margin-bottom: 1.0rem;
    border-bottom: 1px solid var(--border);
  }

  h2 {
    font-size: 1.25rem;
    margin-bottom: 0.5rem;
  }

  .tab-view {
      background-color: var(--background);
      padding: 1rem;
      max-height: 60vh;
      overflow-y: auto;
  }

</style>

<dialog bind:this={dialog} onclose={onClose} oncancel={cancel} onpointerdown={pressAway} transition:dialogFly={{ y: "1rem" }}>

  <div class="header">
    <h2>Settings</h2>
    <button class="icon-btn" aria-label="Close" onclick={onClose}>
      <span class="icon" aria-hidden="true">{@html icon_x}</span>
    </button>
  </div>


{#each tabs as tab, index}
  <button
    class="is-tab"
    class:is-active={current_tab == index}
    onclick={() => current_tab = index}
  >
    {tab.label}
  </button>
{/each}

  <div class="tab-view">
    <Tabs/>
  </div>

</dialog>
