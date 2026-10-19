<script lang="ts">
  import { AnimatePhotos, SendKey } from "@/lib/client/settings.svelte";
  import * as settings from "@/lib/client/settings.svelte";
  import { LABELS_ANIMATE_PHOTOS, LABELS_SEND_KEY, MSG_SAVED, MSG_ERROR, MSG_REMOVE_PUSH, MSG_DISABLE_NOTIFICATION } from "@/lib/labels";
  import * as events from "@/lib/events";
  import * as notifications from "@/lib/client/notifications.svelte";

  // Notifications
  let notifications_notify = $state("");

  // Client Preferences
  let client_theme = $state(settings.getTheme());
  let client_animate = $state(settings.getAnimatePhotos());
  let client_away_minutes = $state<number | null>(settings.getAwayMinutes());
  let client_send_key = $state(settings.getSendKey());
  let client_notify = $state("");

  function savePreferences() {
    settings.setTheme(client_theme);
    settings.setAnimatePhotos(client_animate);
    settings.setSendKey(client_send_key);

    if (client_away_minutes !== null) {
      settings.setAwayMinutes(client_away_minutes);
    }

    // Revert on invalid
    client_away_minutes = settings.getAwayMinutes();

    client_notify = MSG_SAVED;
  }

  async function enableBrowser() {
    const result = await notifications.enableBrowserNotifications();
    notifications_notify = result ? `${MSG_ERROR}: ${result}` : MSG_SAVED;
  }

  // A granted permission can only be withdrawn in the browser's own settings
  function disableBrowser() {
    notifications_notify = MSG_DISABLE_NOTIFICATION;
  }

  async function subscribePush() {
    const result = await notifications.enablePush();
    notifications_notify = result ? `${MSG_ERROR}: ${result}` : MSG_SAVED;
  }

  async function unsubscribePush() {
    const accepted = await events.confirm(MSG_REMOVE_PUSH);

    // If dialog is declined, cancel operation
    if (!accepted) {
      return;
    }

    const result = await notifications.disablePush();
    notifications_notify = result ? `${MSG_ERROR}: ${result}` : MSG_SAVED;
  }
</script>

<div class="section fields">
  <div class="grid-title">
    <h3>Notifications</h3>
  </div>
  <p class="message">{notifications_notify}</p>

  <label for="browser-notify">Browser Notifications</label>
  {#if !notifications.isBrowserNotificationGranted()}
      <button id="browser-notify" onclick={enableBrowser}>Enable</button>
  {:else}
      <button id="browser-notify" onclick={disableBrowser}>Disable</button>
  {/if}

  <label for="push-notify">Push Notifications</label>
  {#if !notifications.isPushSubscribed()}
    <button id="push-notify" onclick={subscribePush}>Enable</button>
  {:else}
    <button id="push-notify" class="is-danger" onclick={unsubscribePush}>
        Disable
    </button>
  {/if}
</div>

<div class="section fields">
  <div class="grid-title">
    <h3>Preferences</h3>
  </div>
  <p class="message">{client_notify}</p>

  <label for="theme">Theme</label>
  <select id="theme" bind:value={client_theme}>
      <option value="dark">Dark</option>
      <option value="light">Light</option>
      <option value="spore">Spore</option>
  </select>

  <label for="animate-photos">Animate Photos</label>
  <select id="animate-photos" bind:value={client_animate}>
      {#each Object.values(AnimatePhotos) as value}
          <option {value}>{LABELS_ANIMATE_PHOTOS[value]}</option>
      {/each}
  </select>

  <label for="send-key">Send Message Key</label>
  <select id="send-key" bind:value={client_send_key}>
      {#each Object.values(SendKey) as value}
          <option {value}>{LABELS_SEND_KEY[value]}</option>
      {/each}
  </select>

  <label for="away-time">Away after (minutes)</label>
  <input id="away-time" name="away-time" type="number" min="0" step="any" bind:value={client_away_minutes}/>

  <button class="full-row" onclick={savePreferences}>Save Changes</button>
</div>
