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
  import { isConfirmVisible, getConfirmMessage, resolveConfirm } from "./lib/confirm.svelte";
  import { loadProfile } from "./lib/profile.svelte";
  import { loadServerInfo } from "./lib/server.svelte";
  import { loadPreferences } from "./lib/preferences.svelte";
  import { loadRooms } from "./lib/rooms/layout.svelte";
  import { getSelectedRoom } from "./lib/session.svelte";
  import { close, connect } from "./lib/socket.svelte";
  import { untrack } from "svelte";
  import { clearMessages } from "./lib/messages.svelte";
  import { clearRoomData } from "./lib/rooms/data.svelte";
  import { clearUsers } from "./lib/users.svelte";
  import { Pane, isOpen } from "./lib/panes.svelte";

  $effect(() => {
    if (getToken() === null) return;

    connect(); // Connect to server's websocket

    // Load server elements sequentially
    loadServerInfo()
      .then(loadProfile)
      .then(loadPreferences)
      .then(loadRooms);

    return close;
  });

  // Removes everything stored for an account once its token is gone
  $effect(() => {
    if (getToken() !== null) return;

    untrack(() => {
      clearMessages();
      clearRoomData();
      clearUsers();
    });
  });
</script>

{#if getToken() !== null}
  <!-- Load main app when token is set -->
  <div class="app">
    <Toolbar/>
    <div class="panes">
      {#if isOpen(Pane.room)}
        <Rooms/>
      {/if}
      {#if isOpen(Pane.members)}
        <Members/>
      {/if}
      <Chat/>
      {#if isOpen(Pane.xbox)}
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
{#if isConfirmVisible()}
  <Popup
    message={getConfirmMessage()}
    onConfirm={() => resolveConfirm(true)}
    onCancel={() => resolveConfirm(false)}
  />
{/if}
