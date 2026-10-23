<script lang="ts">
  import type { Component } from "svelte";
  import { Permission } from "@/bindings/shared";
  import { dialogFly } from "@/lib/display/transitions";
  import { collapsible } from "@/lib/display/collapse";
  import { modal } from "@/lib/display/modal";
  import * as settings from "@/lib/rooms/settings.svelte";
  import General from "./general.svelte";
  import Access from "./access.svelte";
  import icon_x from "@/assets/icons/x.svg?raw";

  interface Props {
    onClose: () => void;
  }

  interface Tab {
    label: string;
    view: Component;
  }

  let { onClose }: Props = $props();
  let current_tab = $state(0);

  const tabs: Tab[] = $derived.by(() => {
    const permissions = settings.getPermissions();
    const shown: Tab[] = [{ label: "General", view: General }];

    // Show access to users who can invite, ban, or grant
    if (
      permissions.includes(Permission.invite)
      || permissions.includes(Permission.ban)
      || permissions.includes(Permission.grant)
    ) {
      shown.push({ label: "Access", view: Access });
    }

    return shown;
  });

  const View = $derived((tabs[current_tab] ?? tabs[0]).view);

  // Clear room ID on close
  $effect(() => {
    return settings.close;
  });

  // Close when room is removed from room list
  $effect(() => {
    if (settings.getRoom() === undefined) {
      onClose();
    }
  });
</script>

<style>
  dialog {
    width: 50rem;
    height: 36rem;
  }
</style>

<dialog class="menu" {@attach modal(onClose)} {@attach collapsible} onclose={onClose} transition:dialogFly={{ y: "1rem" }}>
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

  <div class="tab-panel">
    <View/>
  </div>
</dialog>
