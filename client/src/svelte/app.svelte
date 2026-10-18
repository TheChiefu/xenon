<script lang="ts">
  import Login from "./login.svelte";
  import Popup from "./modals/popup.svelte";
  import NewRoom from "./modals/new_room.svelte";
  import Settings from "./modals/user/settings.svelte";
  import Admin from "./modals/admin/settings.svelte";
  import Profile from "./modals/profile/viewer.svelte";
  import Discover from "./modals/discover.svelte";
  import RoomSettings from "./modals/room/settings.svelte";

  import Toolbar from "./toolbar/top.svelte";
  import BottomToolbar from "./toolbar/bottom.svelte";
  import Container from "./panels/container.svelte";
  import Rooms from "./panels/rooms.svelte";
  import Members from "./panels/members.svelte";
  import Chat from "./chat/area.svelte";
  import Xbox from "./panels/xbox.svelte";
  import ContextMenu from "./context_menu.svelte";

  import * as session from "@/lib/connection/session.svelte";
  import { close, connect } from "@/lib/connection/socket.svelte";
  import * as server from "@/lib/connection/server.svelte";
  import * as profile from "@/lib/users/profile.svelte";
  import * as preferences from "@/lib/client/preferences.svelte";
  import * as layout from "@/lib/rooms/layout.svelte";

  import { Pane } from "@/lib/display/panes.svelte";
  import * as panes from "@/lib/display/panes.svelte";
  import { Modal } from "@/lib/display/modals.svelte";
  import * as modals from "@/lib/display/modals.svelte";
  import * as creation from "@/lib/rooms/creation.svelte";
  import * as profile_viewer from "@/lib/users/profile_viewer.svelte";
  import * as confirm from "@/lib/display/confirm.svelte";
  import { LocalEvent } from "@/lib/events";
  import * as events from "@/lib/events";
  import { COMPACT_CLASS, isCompact } from "@/lib/utils";
  import type { ToolbarButton } from "@/lib/display/toolbar";
  import * as xbox from "@/lib/xbox.svelte";
  import { Platform } from "@/bindings/shared";
  import { ICONS_PLATFORM } from "@/lib/labels";

  import icon_search from "@/assets/icons/search.svg?raw";
  import icon_menu from "@/assets/icons/menu.svg?raw";
  import icon_users from "@/assets/icons/users.svg?raw";

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

  // Close Xbox pane without a linked account
  $effect(() => {
    if (profile.get() === null) return;
    if (!xbox.isLinked()) panes.setOpenState(Pane.xbox, false);
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
        <Container pane={Pane.room}>
          <Rooms/>
        </Container>
      {/if}

      <!-- Room Member List-->
      {#if opened[Pane.members]}
        <Container pane={Pane.members}>
          <Members {roomId} open={panes.isOpen(Pane.members)}/>
        </Container>
      {/if}

      <!-- Chat Box: Always loaded-->
      <Chat {roomId}/>

      <!-- Xbox Navigator-->
      {#if opened[Pane.xbox]}
        <Container pane={Pane.xbox}>
          <Xbox/>
        </Container>
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

  {#if modals.isOpen(Modal.discover)}
    <Discover onClose={modals.close}/>
  {/if}

  {#if modals.isOpen(Modal.room_settings)}
    <RoomSettings onClose={modals.close}/>
  {/if}

  {#if profile_viewer.isOpen()}
    <Profile onClose={profile_viewer.close}/>
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
