<script lang="ts">
  import Identity from "@/components/identity.svelte";
  import ServerInfo from "@/components/server_info.svelte";
  import { Platform } from "@/bindings/shared";
  import { ICONS_PLATFORM } from "@/lib/labels";
  import * as xbox from "@/lib/xbox.svelte";
  import { Pane } from "@/lib/panes.svelte";
  import * as panes from "@/lib/panes.svelte";

  import icon_search from "@/assets/icons/search.svg?raw";
  import icon_menu from "@/assets/icons/menu.svg?raw";
  import icon_users from "@/assets/icons/users.svg?raw";
</script>

<style>
  .toolbar {
    display: flex;
    align-items: center;
    gap: 0.75rem;
    height: 4rem;
    flex: none;
    padding: 0;
    border-bottom: 1px solid var(--border);
    background: var(--component);
  }

  .spacer {
    flex: 1;
  }

  .toolbar-user {
    height: 100%;
  }

  .pane-btn {
    opacity: 0.4;
  }

  .pane-btn.open {
    opacity: 1;
  }

</style>

<header class="toolbar">


  <ServerInfo/>

  <button class="icon-btn" aria-label="Search">
    <span class="icon" aria-hidden="true">{@html icon_search}</span>
  </button>

  <div class="spacer"></div>

  <button class="icon-btn pane-btn" class:open={panes.isOpen(Pane.room)} aria-label="Rooms" title="Rooms" onclick={() => panes.toggle(Pane.room)}>
    <span class="icon" aria-hidden="true">{@html icon_menu}</span>
  </button>

  <button class="icon-btn pane-btn" class:open={panes.isOpen(Pane.members)} aria-label="Members" title="Members" onclick={() => panes.toggle(Pane.members)}>
    <span class="icon" aria-hidden="true">{@html icon_users}</span>
  </button>

  {#if xbox.isLinked()}
    <button class="icon-btn pane-btn" class:open={panes.isOpen(Pane.xbox)} aria-label="Xbox" title="Xbox" onclick={() => panes.toggle(Pane.xbox)}>
      <span class="icon" aria-hidden="true">{@html ICONS_PLATFORM[Platform.xbox]}</span>
    </button>
  {/if}

  <div class="toolbar-user">
    <Identity/>
  </div>

</header>
