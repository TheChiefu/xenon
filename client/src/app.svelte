<script lang="ts">
  import Login from "./views/login.svelte";
  import Toolbar from "./components/areas/toolbar.svelte";
  import { getToken } from "./lib/session.svelte";
</script>

{#if getToken() !== null}
  <div class="app">
    <Toolbar/>
    <div class="panes"></div>
  </div>
{:else}
  <Login/>
{/if}
