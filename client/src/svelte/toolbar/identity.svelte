<script lang="ts">
  import * as session from "@/lib/connection/session.svelte";
  import * as notifications from "@/lib/client/notifications.svelte";
  import { signOut } from "@/lib/api/auth";
  import * as profile from "@/lib/users/profile.svelte";
  import * as users from "@/lib/users/data.svelte";
  import * as status from "@/lib/users/status.svelte";
  import { LABELS_STATUS } from "@/lib/labels";
  import { GlobalRole } from "@/bindings/types/users";
  import { Appearance } from "@/bindings/types/status";
  import { LocalEvent } from "@/lib/events";
  import * as events from "@/lib/events";
  import { dialogFly } from "@/lib/display/transitions";

  import icon_refresh from "@/assets/icons/refresh.svg?raw";
  import icon_gear from "@/assets/icons/gear.svg?raw";
  import icon_shield from "@/assets/icons/shield.svg?raw";

  let menuOpen = $state(false);

  const myStatus = $derived(users.getStatus(profile.get()?.id ?? ""));

  // Messages
  const msg_sign_out = "Are you sure you want to sign out?";

  // Closes the menu and opens the settings
  function openSettings() {
    menuOpen = false;
    events.dispatch(LocalEvent.settings_requested);
  }

  function openAdmin() {
    menuOpen = false;
    events.dispatch(LocalEvent.admin_requested);
  }

  async function doSignout() {
    const result = await events.confirm(msg_sign_out);
    if (result) {

      // Unsubscribe from push notifications
      await notifications.disablePush();

      // Revoke session on server
      const account = session.getLoggedInAccount();
      if (account !== null) {
        try {
          await signOut(account.url, account.token);
        } catch {
          // Failure still signs out
        }
      }

      session.removeLoggedInAccount();
    }
  }

  function onStatusChange(event: Event) {
    if (!(event.target instanceof HTMLSelectElement)) return;

    status.set(event.target.value as Appearance);
  }

  function pressAway(event: PointerEvent) {
  if (event.target instanceof Element && !event.target.closest(".identity")) {
    menuOpen = false;
  }

}
</script>

<style>
  .card {
    --point: 2.5rem;

    display: flex;
    height: 100%;
    clip-path: polygon(
      0 0,                  /* Top Left */
      100% 0,               /* Top Right */
      100% 100%,            /* Bottom Right */
      0 100%,               /* Bottom Left */
      var(--point) 50%      /* Notch */
    );
    background: var(--component-raised);
    color: var(--text);
  }

  .card:hover {
    background-color: var(--component-hover);
  }

  .user,
  .user:hover {
    flex: 1;
    min-width: 0;
    padding: 0;
    border: 0;
    background: none;
    color: inherit;
    font: inherit;
    cursor: pointer;
  }

  .text {
    display: grid;
    padding: 0 0.5rem 0 var(--point);
    text-align: center;
  }

  .frame {
    height: 100%;
    filter: drop-shadow(calc(-1 * var(--status-bar-width)) 0 0 var(--status-color));
  }

  .status-online    { --status-color: var(--status-online); }
  .status-busy      { --status-color: var(--status-busy); }
  .status-away      { --status-color: var(--status-away); }
  .status-offline   { --status-color: var(--status-offline); }

  .user-name, .user-handle {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .user-name {
    font-weight: bold;
  }

  .user-handle {
    color: var(--text-dim);
    font-size: 0.78rem;
  }

  .identity {
    position: relative;
  }

  .menu-clip {
    position: absolute;
    top: 100%;
    right: 0;
    z-index: 10;
    overflow: hidden;
  }

  .menu {
    display: grid;
    gap: 0.5rem;
    width: 16rem;
    padding: 0.75rem;
    border: 1px solid var(--border);
    background: var(--component);
  }

  .tools {
    display: flex;
    justify-content: center;
    gap: 0.5rem;
    padding: 0.25rem;
  }

  .tools .icon-btn {
    border: 1px solid var(--border);
  }

  button:hover {
    background-color: var(--component-hover);
  }

</style>

<svelte:window onpointerdown={pressAway}/>

<div class="identity">
  <div class="frame status-{myStatus.status}">
    <div class="card">
      <button
        class="user"
        aria-expanded={menuOpen}
        onclick={() => menuOpen = !menuOpen}
      >
        <span class="text">
          <span class="user-name">{profile.get()?.display_name ?? ""}</span>
          <span class="user-handle">@{session.getLoggedInAccount()?.username}</span>
        </span>
      </button>
    </div>
  </div>

  {#if menuOpen}
    <div class="menu-clip">
    <div class="menu" transition:dialogFly={{ y: "-1rem" }}>
      <div class="tools">
        <button class="icon-btn" aria-label="Refresh Page" onclick={() => location.reload()} title="Hard refresh web page">
          <span class="icon" aria-hidden="true">{@html icon_refresh}</span>
        </button>
        <button class="icon-btn" aria-label="Settings" onclick={openSettings} title="Open Settings Menu">
          <span class="icon" aria-hidden="true">{@html icon_gear}</span>
        </button>
        {#if profile.get()?.global_role === GlobalRole.admin}
          <button class="icon-btn" aria-label="Admin" onclick={openAdmin} title="Open Admin Menu">
            <span class="icon" aria-hidden="true">{@html icon_shield}</span>
          </button>
        {/if}
      </div>

      <select
        id="status"
        title="How you are shown to others when using the client"
        value={status.getPreferred()}
        onchange={onStatusChange}
      >
        {#each Object.values(Appearance) as value}
          <option {value}>{LABELS_STATUS[value]}</option>
        {/each}
      </select>

      <hr/>

      <button onclick={session.clearCurrentLogin}>Switch Account</button>
      <button class="is-danger" onclick={doSignout}>Sign Out</button>
    </div>
    </div>
  {/if}
</div>