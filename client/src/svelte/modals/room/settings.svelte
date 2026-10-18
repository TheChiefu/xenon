<script lang="ts">
  import type { Component } from "svelte";
  import { Permission } from "@/bindings/shared";
  import { dialogFly } from "@/lib/display/transitions";
  import * as settings from "@/lib/rooms/settings.svelte";
  import General from "./general.svelte";
  import Invites from "./invites.svelte";
  import Bans from "./bans.svelte";
  import icon_x from "@/assets/icons/x.svg?raw";

  interface Props {
    onClose: () => void;
  }

  interface Tab {
    label: string;
    view: Component;
  }

  let { onClose }: Props = $props();
  let dialog = $state<HTMLDialogElement | null>(null);
  let current_tab = $state(0);

  const tabs: Tab[] = $derived.by(() => {
    const permissions = settings.getPermissions();
    const shown: Tab[] = [{ label: "General", view: General }];

    if (permissions.includes(Permission.invite)) {
      shown.push({ label: "Invites", view: Invites });
    }

    if (permissions.includes(Permission.ban)) {
      shown.push({ label: "Bans", view: Bans });
    }

    return shown;
  });

  const View = $derived((tabs[current_tab] ?? tabs[0]).view);

  $effect(() => {
    dialog?.showModal();
  });

  // Close when room is removed from room list
  $effect(() => {
    if (settings.getRoom() === undefined) {
      onClose();
    }
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
    <h2>{settings.getRoom()?.name ?? "Room"} Settings</h2>
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
