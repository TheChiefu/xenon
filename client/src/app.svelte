<script lang="ts">
  import Login from "./views/login.svelte";
  import Toolbar from "./components/areas/toolbar.svelte";
  import Popup from "./views/popup.svelte";
  import Rooms from "./components/areas/rooms.svelte";
  import ContextMenu from "./components/context_menu.svelte";
  import { getToken } from "./lib/session.svelte";
  import { isConfirmVisible, getConfirmMessage, resolveConfirm } from "./lib/confirm.svelte";
  import { loadProfile } from "./lib/profile.svelte";
  import { loadPreferences } from "./lib/preferences.svelte";
  import { loadRooms } from "./lib/rooms/layout.svelte";

  $effect(() => {
    if (getToken() === null) return;

    loadProfile();
    loadPreferences().then(loadRooms);
  });
</script>

{#if getToken() !== null}
  <!-- Load main app when token is set -->
  <div class="app">
    <Toolbar/>
    <div class="panes">
      <Rooms/>
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
