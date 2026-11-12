<script lang="ts">
  import { dialogFly } from "@/lib/display/transitions";
  import { collapsible } from "@/lib/display/collapse";
  import { modal } from "@/lib/display/modal";
  import Codes from "./codes.svelte";
  import Users from "./users.svelte";
  import icon_x from "@/assets/icons/x.svg?raw";

  interface Props {
    onClose: () => void;
  }

  let { onClose }: Props = $props();
  let current_tab = $state(0);

  const tabs = [
    { label: "Invites", view: Codes },
    { label: "Users", view: Users },
  ];

  const View = $derived(tabs[current_tab].view);
</script>

<style>
  dialog {
    width: 40rem;
    height: 36rem;
  }
</style>

<dialog class="menu" {@attach modal(onClose)} {@attach collapsible} onclose={onClose} transition:dialogFly={{ y: "1rem" }}>
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

  <div class="tab-panel">
    <View/>
  </div>
</dialog>
