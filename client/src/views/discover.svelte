<script lang="ts">
  import { Visibility } from "@/bindings/shared";
  import { DESCRIPTIONS_VISIBILITY, LABELS_VISIBILITY, MSG_ERROR } from "@/lib/labels";
  import { dialogFly } from "@/lib/transitions";
  import * as discovery from "@/lib/rooms/discovery.svelte";
  import icon_x from "@/assets/icons/x.svg?raw";

  interface Props {
    onClose: () => void;
  }

  let { onClose }: Props = $props();
  let dialog = $state<HTMLDialogElement | null>(null);

  $effect(() => {
    dialog?.showModal();
  });

  discovery.loadFirstPage();

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
  .view {
    padding: var(--section-padding);
    flex: 1;
    min-height: 0;
    overflow-y: auto;
    background-color: var(--background);
  }

  .notify {
    color: var(--text-dim);
    font-style: italic;
  }

  .rooms {
    display: grid;
    grid-template-columns: 2fr 1fr 1fr auto;
  }

  .status {
    color: var(--text-dim);
  }

  .more {
    margin-top: var(--section-gap);
    align-self: center;
  }
</style>

<dialog class="menu" bind:this={dialog} onclose={onClose} oncancel={cancel} onpointerdown={pressAway} transition:dialogFly={{ y: "1rem" }}>
  <div class="header">
    <h2>Discover Rooms</h2>
    <button class="icon-btn" aria-label="Close" onclick={onClose}>
      <span class="icon" aria-hidden="true">{@html icon_x}</span>
    </button>
  </div>

  <div class="view fill-column">
    <div class="grid-title">
      <h3>Rooms</h3>
      {#if discovery.getError()}
        <span class="notify">{MSG_ERROR}: {discovery.getError()}</span>
      {/if}
    </div>

    <div class="rooms grid table">
      <span class="grid-heading">Name</span>
      <span class="grid-heading">Type</span>
      <span class="grid-heading">Created</span>
      <span class="grid-heading"></span>

      {#each discovery.getRooms() as entry (entry.id)}
        <span>{entry.name}</span>
        <span title={DESCRIPTIONS_VISIBILITY[entry.visibility]}>{LABELS_VISIBILITY[entry.visibility]}</span>
        <span>{new Date(entry.created_at).toLocaleDateString()}</span>
        <span>
          {#if discovery.isJoined(entry.id)}
            <span class="status">Joined</span>
          {:else if entry.visibility === Visibility.public}
            <button onclick={() => discovery.join(entry.id)} disabled={discovery.isPending()}>Join</button>
          {:else}
            <span class="status">Invite Only</span>
          {/if}
        </span>
      {/each}
    </div>

    {#if !discovery.isFinalPage()}
      <button class="more" onclick={discovery.loadPage} disabled={discovery.isPending()}>Load More</button>
    {/if}
  </div>
</dialog>
