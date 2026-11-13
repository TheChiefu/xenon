<script lang="ts">
  import Login from "./login.svelte";
  import Popup from "./modals/popup.svelte";
  import NewRoom from "./modals/new_room.svelte";
  import Settings from "./modals/user/settings.svelte";
  import Admin from "./modals/admin/settings.svelte";
  import Profile from "./modals/profile/viewer.svelte";
  import ImageViewer from "./modals/image_viewer.svelte";
  import Discover from "./modals/discover.svelte";
  import RoomSettings from "./modals/room/settings.svelte";
  import Search from "./modals/search.svelte";

  import Toolbar from "./toolbar/top.svelte";
  import BottomToolbar from "./toolbar/bottom.svelte";
  import Container from "./panes/container.svelte";
  import Rooms from "./panes/rooms.svelte";
  import Active from "./call/active.svelte";
  import Preview from "./call/preview.svelte";
  import Content from "./content.svelte";
  import Inbox from "./panes/inbox.svelte";
  import ContextMenu from "./context_menu.svelte";

  import * as session from "@/lib/connection/session.svelte";
  import { close, connect } from "@/lib/connection/socket.svelte";
  import * as server from "@/lib/connection/server.svelte";
  import * as profile from "@/lib/users/profile.svelte";
  import * as preferences from "@/lib/client/preferences.svelte";
  import * as layout from "@/lib/rooms/layout.svelte";
  import * as invites from "@/lib/rooms/invites.svelte";

  import { Pane } from "@/lib/display/panes.svelte";
  import * as panes from "@/lib/display/panes.svelte";
  import { Modal } from "@/lib/display/modals.svelte";
  import * as modals from "@/lib/display/modals.svelte";
  import * as creation from "@/lib/rooms/creation.svelte";
  import * as profile_viewer from "@/lib/users/profile_viewer.svelte";
  import * as image_viewer from "@/lib/display/image_viewer.svelte";
  import * as confirm from "@/lib/display/confirm.svelte";
  import { COMPACT_CLASS, isCompact } from "@/lib/utils";
  import type { ToolbarButton } from "@/lib/display/toolbar";
  import { Kind } from "@/lib/display/toolbar";
  import { LocalEvent } from "@/lib/events";
  import * as events from "@/lib/events";

  import icon_rooms from "@assets/icons/door-open.svg?raw";
  import icon_call from "@assets/icons/user-broadcast.svg?raw";
  import icon_inbox from "@assets/icons/mail.svg?raw";
  import icon_search from "@assets/icons/search.svg?raw";

  // Room shown by chat and members
  const roomId: string | null = $derived(session.getSelectedRoom());

  // Keep track of which panes are open or not
  const opened: Record<Pane, boolean> = $state({
    [Pane.rooms]: false,
    [Pane.call]: false,
    [Pane.inbox]: false,
  });

  // Pane toggles, laid out by whichever toolbar is shown
  const paneButtons: ToolbarButton[] = $derived([
    {
      label: "Rooms",
      icon: icon_rooms,
      active: panes.isOpen(Pane.rooms),
      kind: Kind.Regular,
      onclick: () => panes.toggle(Pane.rooms),
    },
    {
      label: "Call",
      icon: icon_call,
      active: panes.isOpen(Pane.call),
      kind: Kind.Regular,
      onclick: () => panes.toggle(Pane.call),
    },
    {
      label: "Inbox",
      icon: icon_inbox,
      active: panes.isOpen(Pane.inbox),
      badge: invites.getInvites().length,
      kind: Kind.Overlay,
      onclick: () => panes.toggle(Pane.inbox),
    },
    {
      label: "Search",
      icon: icon_search,
      active: modals.isOpen(Modal.search),
      kind: Kind.Modal,
      onclick: () => events.dispatch(LocalEvent.search_requested),
    },
  ]);

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
      .then(layout.loadRooms)
      .then(invites.load);

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

      <!-- Rooms and Members -->
      {#if opened[Pane.rooms]}
        <Container pane={Pane.rooms}>
          <Rooms {roomId} open={panes.isOpen(Pane.rooms)}/>
        </Container>
      {/if}

      <!-- Call -->
      {#if opened[Pane.call]}
        <Container pane={Pane.call}>
          <Preview {roomId}/>
          <Active/>
        </Container>
      {/if}

      <!-- Chat and Stream: Always loaded-->
      <Content {roomId}/>

      {#if opened[Pane.inbox]}
        <Container pane={Pane.inbox}>
          <Inbox/>
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

  {#if modals.isOpen(Modal.search)}
    <Search onClose={modals.close}/>
  {/if}

  {#if profile_viewer.isOpen()}
    <Profile onClose={profile_viewer.close}/>
  {/if}

  {#if image_viewer.isOpen()}
    <ImageViewer onClose={image_viewer.close}/>
  {/if}

{/if}

<!-- Right click menu, filled by whichever element was right clicked -->
<ContextMenu/>

<!-- Choice popup setter for generic popup messages -->
{#if confirm.isVisible()}
  <Popup
    message={confirm.getMessage()}
    choices={confirm.getChoices()}
    onChoose={(choice) => confirm.resolve(choice)}
    onCancel={() => confirm.resolve(null)}
  />
{/if}
