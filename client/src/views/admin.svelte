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
  .tab-view {
    padding: var(--section-padding);
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    background-color: var(--background);
  }
</style>

<dialog class="menu" bind:this={dialog} onclose={onClose} oncancel={cancel} onpointerdown={pressAway} transition:dialogFly={{ y: "1rem" }}>
  <div class="header">
    <h2>Admin</h2>
    <button class="icon-btn" aria-label="Close" onclick={onClose}>
      <span class="icon" aria-hidden="true">{@html icon_x}</span>
    </button>
  </div>

  <div>
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
</dialog>
