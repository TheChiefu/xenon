<script lang="ts">
  import type { ProfilePatch } from "@/bindings/routes/users";
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
  let error = $state("");
  let current_tab = $state(0);

  // User Credentials
  let username = $state("");
  let password = $state("");
  let display_name = $state("");
  let invite_code = $state("");
  let linked_xbox = $state(false);
  let linked_steam = $state(false);

  // Messages
  const msg_remove_push = "Are you sure you want to disable push notifications?";
  const msg_sign_out = "Are you sure you want to sign out?";
  const msg_disable_notification = "Browser notifications must be revoked via browser settings.";

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

  async function doSignout(){
    const result = await confirmDialog(msg_sign_out);
    if (result) {
      signOut();
    }
  }

  // Returns "is-active" if the given tab index is the current one
  function tabClass(tab: number): string {
    return current_tab == tab ? 'is-active' : '';
  }

</script>

<style>
  dialog {
    width: 40rem;
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

  textarea {
      min-height: 10rem;
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

  .tab-view {
      background-color: var(--background);
      padding: 1rem;
  }

  .tab-btn {
      border: none;
      background: var(--btn-tab);
  }

</style>

<dialog bind:this={dialog} onclose={onClose}>

  <div class="header">
    <h2>Settings</h2>
    <button class="icon-btn" aria-label="Close" onclick={() => dialog?.close()}>
      <span class="icon" aria-hidden="true">{@html icon_x}</span>
    </button>
  </div>


<button class="is-tab tab-btn {tabClass(0)}" onclick={() => current_tab = 0}>Client</button>
<button class="is-tab tab-btn {tabClass(1)}" onclick={() => current_tab = 1}>User</button>

  <!-- Client Settings -->
  <div class="tab-view">
  {#if current_tab == 0}
      <div class="rows">
        <h3 class="full-row">Notifications</h3>

        <p>Browser Notifications:</p>
        {#if !isBrowserNotificationGranted()}
            <button onclick={() => tryRun(enableBrowserNotifications)}>Enable</button>
        {:else}
            <button onclick={() => error = msg_disable_notification}>Disable</button>
        {/if}

        <p>Push Notifications:</p>
        {#if !isPushSubscribed()}
          <button id="push-notify" onclick={() => tryRun(enablePush)}>Enable</button>
        {:else}
          <button id="push-notify" class="is-danger" onclick={() => tryRun(disablePush, msg_remove_push)}>
              Disable
          </button>
        {/if}

        <h3 class="full-row">User</h3>

        <p>Theme:</p>
        <select value={getTheme()} onchange={changeTheme}>
            <option value="dark">Dark</option>
            <option value="light">Light</option>
            <option value="spore">Spore</option>
        </select>

        <p>Animate Photos:</p>
        <select>
            <option value="always">Always</option>
            <option value="on-hover">On Hover</option>
            <option value="never">Never</option>
        </select>

        <hr class="full-row"/>

        <button onclick={doSignout} class="full-row is-danger">Sign Out</button>
      </div>
  {:else if current_tab == 1}

    <!-- User Settings -->
    <div class="rows">
        <h3 class="full-row">Profile</h3>

        <p>Display Name</p>
        <input name="display-name" placeholder="Name visible to others" bind:value={display_name}/>
        <p>Password</p>
        <input name="password" type="password" placeholder="Leave empty if not changing" bind:value={password}/>
        <p>Email</p>
        <input name="email" type="email" placeholder="Not Yet Implemented" disabled/>
        <p>Description</p>
        <textarea name="description"></textarea>
        <p>Link / Unlink Accounts</p>
        <div>
            {#if linked_xbox}
                <button class="is-danger">Unlink Xbox</button>
            {:else}
                <button>Link Xbox</button>
            {/if}
            {#if linked_steam}
                <button class="is-danger">Unlink Steam</button>
            {:else}
                <button>Link Steam</button>
            {/if}
        </div>

        <hr class="full-row"/>
        <h3 class="full-row">Presentation</h3>

        <p>Status</p>
        <select title="How you are shown to others when using the client">
            <option value="online">Online</option>
            <option value="dnd">Do Not Disturb</option>
            <option value="away">Away</option>
            <option value="invisible">Invisible</option>
        </select>

        <p>Away after (minutes)</p>
        <input name="away-time" type="number" placeholder="10"/>

        <hr class="full-row"/>
        <h3 class="full-row" style="color: var(--danger);">Account Deletion</h3>
        <div>
            <label for="delete-username">Release Username</label>
            <input id="delete-username" type="checkbox"/>
        </div>
        <div>
            <label for="delete-messages">Delete All Messages</label>
            <input id="delete-messages" type="checkbox"/>
        </div>
        <button class="full-row is-danger">Delete Account</button>

        <hr class="full-row"/>
        <button class="full-row">Save Changes</button>
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
