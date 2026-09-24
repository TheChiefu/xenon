<script lang="ts">
  import Login from "./views/login.svelte";
  import Toolbar from "./components/areas/toolbar.svelte";
  import Popup from "./views/popup.svelte";
  import { getToken, getUrl } from "./lib/session.svelte";
  import { isConfirmVisible, getConfirmMessage, resolveConfirm } from "./lib/confirm.svelte";
  import { loadProfile } from "./lib/profile.svelte";

  $effect(() => {
    const url = getUrl();
    const token = getToken();
    if (url === null || token === null) return;

    loadProfile(url, token);
  });
</script>

{#if getToken() !== null}
  <div class="app">
    <Toolbar/>
    <div class="panes"></div>
  </div>
{:else}
  <Login/>
{/if}

{#if isConfirmVisible()}
  <Popup
    message={getConfirmMessage()}
    onConfirm={() => resolveConfirm(true)}
    onCancel={() => resolveConfirm(false)}
  />
{/if}
