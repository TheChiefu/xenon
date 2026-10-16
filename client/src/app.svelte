<script lang="ts">
  import Login from "./views/login.svelte";
  import Toolbar from "./components/areas/toolbar.svelte";
  import Popup from "./views/popup.svelte";
  import Rooms from "./components/areas/rooms.svelte";
  import Members from "./components/areas/members.svelte";
  import Chat from "./components/areas/chat.svelte";
  import Xbox from "./components/areas/xbox.svelte";
  import ContextMenu from "./components/context_menu.svelte";
  import { getToken } from "./lib/session.svelte";
  import * as confirm from "./lib/confirm.svelte";
  import * as profile from "./lib/profile.svelte";
  import * as server from "./lib/server.svelte";
  import * as preferences from "./lib/preferences.svelte";
  import * as layout from "./lib/rooms/layout.svelte";
  import { getSelectedRoom } from "./lib/session.svelte";
  import { close, connect } from "./lib/socket.svelte";
  import { untrack } from "svelte";
  import * as messages from "./lib/messages.svelte";
  import * as room from "./lib/rooms/data.svelte";
  import * as users from "./lib/users.svelte";
  import * as members from "./lib/rooms/members.svelte";
  import { Pane } from "./lib/panes.svelte";
  import * as panes from "./lib/panes.svelte";

  $effect(() => {
    if (getToken() === null) return;

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
    if (getToken() !== null) return;

    untrack(() => {
      messages.clear();
      room.clear();
      users.clear();
      members.clear();
    });
  });
</script>

{#if getToken() !== null}
  <!-- Load main app when token is set -->
  <div class="app">
    <Toolbar/>
    <div class="panes">
      {#if panes.isOpen(Pane.room)}
        <Rooms/>
      {/if}
      {#if panes.isOpen(Pane.members)}
        <Members/>
      {/if}
      <Chat/>
      {#if panes.isOpen(Pane.xbox)}
        <Xbox/>
      {/if}
    </div>
  </div>
{:else}
  <!-- If there is no token, show login screen -->
  <Login/>
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
