<script lang="ts">
  import { signOut } from "@/lib/session.svelte";
  import { getTheme, setTheme } from "@/lib/settings.svelte";
  import icon_x from "@/assets/icons/x.svg?raw";

  interface Props {
    onClose: () => void;
  }
  let { onClose }: Props = $props();

  let dialog = $state<HTMLDialogElement | null>(null);

  let browser_notifications = $state(false);
  let push_notifications = $state(false);
  let is_admin = $state(false);

  $effect(() => {
    dialog?.showModal();
  });

  function changeTheme(event: Event) {
    if (event.target instanceof HTMLSelectElement) {
      setTheme(event.target.value);
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

  .settings-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    margin-bottom: 1.0rem;
    border-bottom: 1px solid var(--border);
  }

  .settings-head h2 {
    font-size: 1.25rem;
  }

  .rows {
      width: 100%;
      display: flex;
      flex-direction: column;
  }

</style>

<dialog bind:this={dialog} onclose={onClose}>

  <div class="settings-head">
    <h2>Client Settings</h2>
    <button class="icon-btn" aria-label="Close" onclick={() => dialog?.close()}>
      <span class="icon" aria-hidden="true">{@html icon_x}</span>
    </button>
  </div>

  <h3>Notifications</h3>
  <br/>
  <div class="rows">
    {#if !browser_notifications}
        <button>Turn On Brower Notifications</button>
    {:else}
        <button>Turn Off Brower Notifications</button>
    {/if}

    {#if !push_notifications}
        <button>Turn On Push Notifications</button>
    {:else}
        <button>Turn Off Push Notifications</button>
    {/if}

    {#if is_admin}
        <h3>Admin</h3>
        <button>Create Registration Code</button>
    {/if}
  </div>

  <br/>
  <h3>User</h3>
  <br/>
  <div class="rows">

    <div> <!-- Theme -->
        <label for="theme">Theme:</label>
        <select id="theme" value={getTheme()} onchange={changeTheme}>
            <option value="dark">Dark</option>
            <option value="light">Light</option>
        </select>
    </div>

    <button onclick={signOut}>Sign Out</button>
  </div>

</dialog>
