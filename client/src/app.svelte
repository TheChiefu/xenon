<script lang="ts">
  import { untrack } from "svelte";

  import Login from "./views/login.svelte";
  import Popup from "./views/popup.svelte";
  import NewRoom from "./views/new_room.svelte";
  import Settings from "./views/settings.svelte";
  import Admin from "./views/admin.svelte";

  import Toolbar from "./components/areas/toolbar.svelte";
  import BottomToolbar from "./components/toolbar/bottom_toolbar.svelte";
  import PaneArea from "./components/pane.svelte";
  import Rooms from "./components/areas/rooms.svelte";
  import Members from "./components/areas/members.svelte";
  import Chat from "./components/areas/chat.svelte";
  import Xbox from "./components/areas/xbox.svelte";
  import ContextMenu from "./components/context_menu.svelte";

  import * as session from "./lib/session.svelte";
  import { close, connect } from "./lib/socket.svelte";
  import * as server from "./lib/server.svelte";
  import * as profile from "./lib/profile.svelte";
  import * as preferences from "./lib/preferences.svelte";
  import * as layout from "./lib/rooms/layout.svelte";

  import * as messages from "./lib/messages.svelte";
  import * as room from "./lib/rooms/data.svelte";
  import * as users from "./lib/users.svelte";
  import * as members from "./lib/rooms/members.svelte";

  import { Pane } from "./lib/panes.svelte";
  import * as panes from "./lib/panes.svelte";
  import { Modal } from "./lib/modals.svelte";
  import * as modals from "./lib/modals.svelte";
  import * as creation from "./lib/rooms/creation.svelte";
  import * as confirm from "./lib/confirm.svelte";
  import { COMPACT_CLASS, isCompact } from "./lib/utils";
  import type { ToolbarButton } from "./lib/toolbar";
  import * as xbox from "./lib/xbox.svelte";
  import { Platform } from "./bindings/shared";
  import { ICONS_PLATFORM } from "./lib/labels";

  import icon_search from "./assets/icons/search.svg?raw";
  import icon_menu from "./assets/icons/menu.svg?raw";
  import icon_users from "./assets/icons/users.svg?raw";

  // Room shown by chat and members
  const roomId: string | null = $derived(session.getSelectedRoom());

  // Keep track of which panes are open or not
  const opened: Record<Pane, boolean> = $state({
    [Pane.search]: false,
    [Pane.room]: false,
    [Pane.members]: false,
    [Pane.xbox]: false,
  });

  // Pane toggles, laid out by whichever toolbar is shown
  const paneButtons: ToolbarButton[] = $derived.by(() => {
    const buttons: ToolbarButton[] = [
      {
        pane: Pane.search,
        label: "Search",
        icon: icon_search,
        active: panes.isOpen(Pane.search),
        onclick: () => {},
      },
      {
        pane: Pane.room,
        label: "Rooms",
        icon: icon_menu,
        active: panes.isOpen(Pane.room),
        onclick: () => panes.toggle(Pane.room),
      },
      {
        pane: Pane.members,
        label: "Members",
        icon: icon_users,
        active: panes.isOpen(Pane.members),
        onclick: () => panes.toggle(Pane.members),
      },
    ];

    // Xbox only once an account is linked
    if (xbox.isLinked()) {
      buttons.push({
        pane: Pane.xbox,
        label: "Xbox",
        icon: ICONS_PLATFORM[Platform.xbox],
        active: panes.isOpen(Pane.xbox),
        onclick: () => panes.toggle(Pane.xbox),
      });
    }

    return buttons;
  });

  // Open loaded panes on their open/closed state
  $effect.pre(() => {
    for (const pane of Object.values(Pane)) {
      if (panes.isOpen(pane)) opened[pane] = true;
    }
  });

  // Check window dimensions for resizing
  $effect(() => {
    document.documentElement.classList.toggle(
      COMPACT_CLASS, isCompact.current
    );
  });

  // Start server loads
  $effect(() => {
    if (session.getToken() === null) return;

    connect(); // Connect to server's websocket

    // Load server elements sequentially
    server.loadInfo()
      .then(profile.load)
      .then(preferences.load)
      .then(layout.loadRooms);

    return close;
  });

  // Removes everything stored for an account once its token is gone
  $effect(() => {
    if (session.getToken() !== null) return;

    untrack(() => {
      messages.clear();
      room.clear();
      users.clear();
      members.clear();
      modals.close();
      creation.close();
    });
  });
</script>


<!-- Show main app on token, otherwise login screen -->
{#if session.getToken() === null}  
  <Login/>
{:else}

  <!-- Main App Layout-->
  <div class="app">
    <Toolbar buttons={isCompact.current ? [] : paneButtons}/>

    <!-- Togglable Panes -->
    <div class="panes">

      <!-- Room Navigator -->
      {#if opened[Pane.room]}
        <PaneArea pane={Pane.room}>
          <Rooms/>
        </PaneArea>
      {/if}

      <!-- Room Member List-->
      {#if opened[Pane.members]}
        <PaneArea pane={Pane.members}>
          <Members {roomId} open={panes.isOpen(Pane.members)}/>
        </PaneArea>
      {/if}

      <!-- Chat Box: Always loaded-->
      <Chat {roomId}/>

      <!-- Xbox Navigator-->
      {#if opened[Pane.xbox]}
        <PaneArea pane={Pane.xbox}>
          <Xbox/>
        </PaneArea>
      {/if}
    </div>

    <!-- Bottom Toolbar for narrow views (ie. mobile)-->
    {#if isCompact.current}
      <BottomToolbar buttons={paneButtons}/>
    {/if}

  </div>

  <!-- Modals -->
  {#if creation.isOpen()}
    <NewRoom onClose={creation.close}/>
  {/if}

  {#if modals.isOpen(Modal.settings)}
    <Settings onClose={modals.close}/>
  {/if}

  {#if modals.isOpen(Modal.admin)}
    <Admin onClose={modals.close}/>
  {/if}

{/if}

<!-- Right click menu, filled by whichever element was right clicked -->
<ContextMenu/>

<!-- Yes/No popup setter for generic popup messages -->
{#if confirm.isVisible()}
  <Popup
    message={confirm.getMessage()}
    onConfirm={() => confirm.resolve(true)}
    onCancel={() => confirm.resolve(false)}
  />
{/if}
