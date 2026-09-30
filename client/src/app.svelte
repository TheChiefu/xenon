<script lang="ts">
  import Login from "./views/login.svelte";
  import Toolbar from "./components/areas/toolbar.svelte";
  import Popup from "./views/popup.svelte";
  import Rooms from "./components/areas/rooms.svelte";
  import { getToken, getUrl } from "./lib/session.svelte";
  import { isConfirmVisible, getConfirmMessage, resolveConfirm } from "./lib/confirm.svelte";
  import { loadProfile } from "./lib/profile.svelte";
  import { loadPreferences } from "./lib/preferences.svelte";
  import { loadRooms } from "./lib/rooms/layout.svelte";

  $effect(() => {
    const url = getUrl();
    const token = getToken();
    if (url === null || token === null) return;

    // Load after token is successfully set
    loadProfile(url, token);
    loadPreferences(url, token);
    loadRooms(url, token);
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

<!-- Yes/No popup setter for generic popup messages -->
{#if isConfirmVisible()}
  <Popup
    message={getConfirmMessage()}
    onConfirm={() => resolveConfirm(true)}
    onCancel={() => resolveConfirm(false)}
  />
{/if}
