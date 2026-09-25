<script lang="ts">
  import type { ProfilePatch } from "@/bindings/routes/users";
  import { dialogFly } from "@/lib/transitions";
  import ProfileEditor from "@/components/profile_editor.svelte";
  import { getTheme, setTheme } from "@/lib/settings.svelte";
  import { confirmDialog } from "@/lib/confirm.svelte";
  import { isBrowserNotificationGranted, isPushSubscribed, enableBrowserNotifications, enablePush, disablePush } from "@/lib/notifications.svelte";
  import icon_x from "@/assets/icons/x.svg?raw";

  interface Props {
    onClose: () => void;
  }
  let { onClose }: Props = $props();
  let dialog = $state<HTMLDialogElement | null>(null);
  let error = $state("");
  let current_tab = $state(0);

  // User Credentials
  let username = $state("");
  let password = $state("");
  let invite_code = $state("");
  let linked_xbox = $state(false);
  let linked_steam = $state(false);

  // Messages
  const msg_remove_push = "Are you sure you want to disable push notifications?";
  const msg_disable_notification = "Browser notifications must be revoked via browser settings.";
  const msg_anonymize = "Replaces your display name and releases your username for someone else to take.";

  $effect(() => {
    dialog?.showModal();
  });

  function changeTheme(event: Event) {
    if (event.target instanceof HTMLSelectElement) {
      setTheme(event.target.value);
    }
  }

  // Runs an action, showing its error message if it returns one
  async function tryRun(
    action: () => Promise<string | undefined>,
    message: string = ""
  ) {
    error = "";

    // Optional message
    if (message) {
      const result = await confirmDialog(message);

      // If dialog is declined, cancel operation
      if (!result) {
        return;
      }
    }

    // Perform action
    const result = await action();
    if (result) {
      error = result;
    }
  }

  // Closes on Escape with the fade out
  function cancel(event: Event) {
    event.preventDefault();
    onClose();
  }

  // Returns "is-active" if the given tab index is the current one
  function tabClass(tab: number): string {
    return current_tab == tab ? '': 'is-active';
  }

</script>

<style>
  dialog {
    width: 50rem;
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

  .section {
    border: 1px solid var(--border);
    padding: 1rem;
    margin-bottom: 1rem;
  }

  .error {
    color: var(--danger)
  }

  .tab-view {
      background-color: var(--background);
      padding: 1rem;
      max-height: 60vh;
      overflow-y: auto;
  }

  .tab-btn {
      border: none;
  }


</style>

<dialog bind:this={dialog} onclose={onClose} oncancel={cancel} transition:dialogFly={{ y: "1rem" }}>

  <div class="header">
    <h2>Settings</h2>
    <button class="icon-btn" aria-label="Close" onclick={onClose}>
      <span class="icon" aria-hidden="true">{@html icon_x}</span>
    </button>
  </div>


<button class="is-tab tab-btn {tabClass(0)}" onclick={() => current_tab = 0}>Profile</button>
<button class="is-tab tab-btn {tabClass(1)}" onclick={() => current_tab = 1}>Client</button>
<button class="is-tab tab-btn {tabClass(2)}" onclick={() => current_tab = 2}>Account</button>

  <div class="tab-view">
  {#if current_tab == 0}

    <!-- Profile Settings -->
    <ProfileEditor/>

  {:else if current_tab == 1}

      <!-- Client Settings -->
      <div class="section rows">
        <h3 class="full-row">Notifications</h3>

        <p>Browser Notifications</p>
        {#if !isBrowserNotificationGranted()}
            <button onclick={() => tryRun(enableBrowserNotifications)}>Enable</button>
        {:else}
            <button onclick={() => error = msg_disable_notification}>Disable</button>
        {/if}

        <p>Push Notifications</p>
        {#if !isPushSubscribed()}
          <button id="push-notify" onclick={() => tryRun(enablePush)}>Enable</button>
        {:else}
          <button id="push-notify" class="is-danger" onclick={() => tryRun(disablePush, msg_remove_push)}>
              Disable
          </button>
        {/if}
      </div>

      <div class="section rows">
        <h3 class="full-row">Preferences</h3>

        <p>Theme</p>
        <select value={getTheme()} onchange={changeTheme}>
            <option value="dark">Dark</option>
            <option value="light">Light</option>
            <option value="spore">Spore</option>
        </select>

        <p>Animate Photos</p>
        <select>
            <option value="always">Always</option>
            <option value="on-hover">On Hover</option>
            <option value="never">Never</option>
        </select>

        <p>Away after (minutes)</p>
        <input name="away-time" type="number" placeholder="10"/>
      </div>
  {:else if current_tab == 2}

    <!-- Account Settings -->
    <div class="section rows">
        <h3 class="full-row">Credentials</h3>

        <p>Password</p>
        <input name="password" type="password" placeholder="Leave empty if not changing" bind:value={password}/>
        <p>Email</p>
        <input name="email" type="email" placeholder="Not Yet Implemented" disabled/>
        <label for="revoke-sessions">Sign Out Other Sessions</label>
        <input id="revoke-sessions" type="checkbox"/>
        <button class="full-row">Save Changes</button>
    </div>

    <div class="section rows">
        <h3 class="full-row">Linked Accounts</h3>

        <p>Xbox</p>
        <div class="rows">
          {#if linked_xbox}
              <button class="is-danger">Unlink</button>
              <input placeholder="My Gamertag" disabled/>
          {:else}
              <button>Link</button>
              <input disabled/>
          {/if}
        </div>

        <p>Steam</p>
        <div class="rows">
          {#if linked_steam}
              <button class="is-danger">Unlink</button>
              <input placeholder="My Steam Name" disabled/>
          {:else}
              <button>Link</button>
              <input disabled/>
          {/if}
        </div>

    </div>

    <div class="section rows">
        <h3 class="full-row" style="color: var(--danger);">Account Deletion</h3>
        <label for="delete-username" title={msg_anonymize}>Anonymize</label>
        <input id="delete-username" type="checkbox" title={msg_anonymize}/>
        <label for="delete-messages">Delete All Messages</label>
        <input id="delete-messages" type="checkbox"/>
        <button class="full-row is-danger">Delete Account</button>
    </div>

  {:else}
    <p>Unknown tab</p>
  {/if}
  </div>

  {#if error != ""}
    <hr class="full-row"/>
    <p class="error full-row">{error}</p>
  {/if}

</dialog>
