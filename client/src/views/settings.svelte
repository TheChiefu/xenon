<script lang="ts">
  import { signOut } from "@/lib/session.svelte";
  import { getTheme, setTheme } from "@/lib/settings.svelte";
  import { confirmDialog } from "@/lib/confirm.svelte";
  import { isBrowserNotificationGranted, isPushSubscribed, enableBrowserNotifications, enablePush, disablePush } from "@/lib/notifications.svelte";
  import icon_x from "@/assets/icons/x.svg?raw";

  interface Props {
    onClose: () => void;
  }
  let { onClose }: Props = $props();

  let dialog = $state<HTMLDialogElement | null>(null);

  let is_admin = $state(false);
  let error = $state("");

  $effect(() => {
    dialog?.showModal();
  });

  function changeTheme(event: Event) {
    if (event.target instanceof HTMLSelectElement) {
      setTheme(event.target.value);
    }
  }

  // Runs an action, showing its error message if it returns one
  async function tryRun(action: () => Promise<string | undefined>) {
    const result = await action();
    if (result) {
      error = result;
    }
  }

  async function doSignout(){
    const result = await confirmDialog("Are you sure you want to sign out?");
    if (result) {
      signOut();
    }
  }

</script>

<style>
  dialog {
    width: 30rem;
    padding: 1.5rem;
    border: 1px solid var(--border);
    background: var(--component);
    color: var(--text);
  }

  dialog::backdrop {
    background: var(--modal-background)
  }

  .header {
    display: flex;
    justify-content: space-between;
    margin-bottom: 1.0rem;
    border-bottom: 1px solid var(--border);
  }

  h2, h3 {
    margin-bottom: 0.5rem;
  }

  h2 {
    font-size: 1.25rem;
  }

  .rows {
    display: grid;
    grid-template-columns: auto 1fr;
    width: 100%;
    gap: 0.5rem;
    align-items: center;
    column-gap: 1.5rem;
  }

  .full-row {
    grid-column: 1 / -1;
  }

  .error {
    color: var(--danger)
  }

</style>

<dialog bind:this={dialog} onclose={onClose}>

  <div class="header">
    <h2>Client Settings</h2>
    <button class="icon-btn" aria-label="Close" onclick={() => dialog?.close()}>
      <span class="icon" aria-hidden="true">{@html icon_x}</span>
    </button>
  </div>

  <!-- Contents -->
  <div class="rows">
    <h3 class="full-row">Notifications</h3>

    <p>Browser Notifications:</p>
    {#if !isBrowserNotificationGranted()}
        <button onclick={() => tryRun(enableBrowserNotifications)}>Enable</button>
    {:else}
        <button disabled={true} aria-hidden="true">Enabled</button>
    {/if}

    <p>Push Notifications:</p>
    {#if !isPushSubscribed()}
      <button onclick={() => tryRun(enablePush)}>Enable</button>
    {:else}
      <button onclick={() => tryRun(disablePush)}>Disable</button>
    {/if}

    {#if is_admin}
        <h3 class="full-row">Admin</h3>
        <button class="full-row">Create Registration Code</button>
    {/if}

    <h3 class="full-row">User</h3>

    <label for="theme">Theme:</label>
    <select id="theme" value={getTheme()} onchange={changeTheme}>
        <option value="dark">Dark</option>
        <option value="light">Light</option>
    </select>

    <button onclick={doSignout} class="full-row is-danger">Sign Out</button>

    {#if error != ""}
      <hr class="full-row"/>
      <p class="error full-row">{error}</p>
    {/if}
  </div>

</dialog>
