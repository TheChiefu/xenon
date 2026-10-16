<script lang="ts">
  import { getTheme, setTheme, AnimatePhotos, getAnimatePhotos, setAnimatePhotos, getAwayMinutes, setAwayMinutes } from "@/lib/settings.svelte";
  import { LABELS_ANIMATE_PHOTOS, MSG_SAVED, MSG_ERROR, MSG_REMOVE_PUSH, MSG_DISABLE_NOTIFICATION } from "@/lib/labels";
  import * as confirm from "@/lib/confirm.svelte";
  import { isBrowserNotificationGranted, isPushSubscribed, enableBrowserNotifications, enablePush, disablePush } from "@/lib/notifications.svelte";

  // Notifications
  let notifications_notify = $state("");

  // Client Preferences
  let client_theme = $state(getTheme());
  let client_animate = $state(getAnimatePhotos());
  let client_away_minutes = $state<number | null>(getAwayMinutes());
  let client_notify = $state("");

  function savePreferences() {
    setTheme(client_theme);
    setAnimatePhotos(client_animate);

    if (client_away_minutes !== null) {
      setAwayMinutes(client_away_minutes);
    }

    // Revert on invalid
    client_away_minutes = getAwayMinutes();

    client_notify = MSG_SAVED;
  }

  async function enableBrowser() {
    const result = await enableBrowserNotifications();
    notifications_notify = result ? `${MSG_ERROR}: ${result}` : MSG_SAVED;
  }

  // A granted permission can only be withdrawn in the browser's own settings
  function disableBrowser() {
    notifications_notify = MSG_DISABLE_NOTIFICATION;
  }

  async function subscribePush() {
    const result = await enablePush();
    notifications_notify = result ? `${MSG_ERROR}: ${result}` : MSG_SAVED;
  }

  async function unsubscribePush() {
    const accepted = await confirm.dialog(MSG_REMOVE_PUSH);

    // If dialog is declined, cancel operation
    if (!accepted) {
      return;
    }

    const result = await disablePush();
    notifications_notify = result ? `${MSG_ERROR}: ${result}` : MSG_SAVED;
  }
</script>

<div class="section rows">
  <div class="grid-title">
    <h3>Notifications</h3>
  </div>
  <p class="message">{notifications_notify}</p>

  <p>Browser Notifications</p>
  {#if !isBrowserNotificationGranted()}
      <button onclick={enableBrowser}>Enable</button>
  {:else}
      <button onclick={disableBrowser}>Disable</button>
  {/if}

  <p>Push Notifications</p>
  {#if !isPushSubscribed()}
    <button id="push-notify" onclick={subscribePush}>Enable</button>
  {:else}
    <button id="push-notify" class="is-danger" onclick={unsubscribePush}>
        Disable
    </button>
  {/if}
</div>

<div class="section rows">
  <div class="grid-title">
    <h3>Preferences</h3>
  </div>
  <p class="message">{client_notify}</p>

  <p>Theme</p>
  <select bind:value={client_theme}>
      <option value="dark">Dark</option>
      <option value="light">Light</option>
      <option value="spore">Spore</option>
  </select>

  <p>Animate Photos</p>
  <select bind:value={client_animate}>
      {#each Object.values(AnimatePhotos) as value}
          <option {value}>{LABELS_ANIMATE_PHOTOS[value]}</option>
      {/each}
  </select>

  <p>Away after (minutes)</p>
  <input name="away-time" type="number" min="0" step="any" bind:value={client_away_minutes}/>

  <button class="full-row" onclick={savePreferences}>Save Changes</button>
</div>
