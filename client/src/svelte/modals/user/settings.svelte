<script lang="ts">
  import { dialogFly } from "@/lib/display/transitions";
  import { modal } from "@/lib/display/modal";
  import Profile from "./profile.svelte";
  import Client from "./client.svelte";
  import Account from "./account.svelte";
  import Links from "./links.svelte";
  import icon_x from "@/assets/icons/x.svg?raw";

  interface Props {
    onClose: () => void;
  }

  let { onClose }: Props = $props();
  let current_tab = $state(0);

  const tabs = [
    { label: "Profile", view: Profile },
    { label: "Client", view: Client },
    { label: "Account", view: Account },
    { label: "Links", view: Links },
  ];

  const Tabs = $derived(tabs[current_tab].view);
</script>

<style>
  dialog {
    width: 32rem;
    height: 36rem;
  }
</style>

<dialog class="menu" {@attach modal(onClose)} onclose={onClose} transition:dialogFly={{ y: "1rem" }}>

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

  <div class="tab-panel">
    <Tabs/>
  </div>

</dialog>
