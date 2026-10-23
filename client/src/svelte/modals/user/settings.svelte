<script lang="ts">
  import { dialogFly } from "@/lib/display/transitions";
  import Profile from "./profile.svelte";
  import Client from "./client.svelte";
  import Account from "./account.svelte";
  import Links from "./links.svelte";
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
    { label: "Links", view: Links },
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
    width: 32rem;
    height: 36rem;
  }

  .tab-view {
      background-color: var(--background);
      padding: var(--section-padding);
      flex: 1;
      min-height: 0;
      overflow-y: auto;
  }

</style>

<dialog class="menu" bind:this={dialog} onclose={onClose} oncancel={cancel} onpointerdown={pressAway} transition:dialogFly={{ y: "1rem" }}>

  <div class="header">
    <h2>Settings</h2>
    <button class="icon-btn" aria-label="Close" onclick={onClose}>
      <span class="icon" aria-hidden="true">{@html icon_x}</span>
    </button>
  </div>


  <div>
    {#each tabs as tab, index}
      <button
        class="is-tab"
        class:is-active={current_tab == index}
        onclick={() => current_tab = index}
      >
        {tab.label}
      </button>
    {/each}
  </div>

  <div class="tab-view">
    <Tabs/>
  </div>

</dialog>
