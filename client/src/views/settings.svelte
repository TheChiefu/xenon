<script lang="ts">
    import { signOut } from "@/lib/session.svelte";
    import { getTheme, setTheme } from "@/lib/settings.svelte";
    import icon_x from "@/assets/icons/x.svg?raw";

    interface Props {
      onClose: () => void;
    }
    let { onClose }: Props = $props();

    let browser_notifications = $state(false);
    let push_notifications = $state(false);
    let is_admin = $state(false);

    function toggleBrowserNotifications() {
      browser_notifications = !browser_notifications;
    }

    function enablePushNotifications() {
      // Perform VAPID process
      push_notifications = true;
    }

    function disablePushNotifications() {
      // Perform disable VAPID process
      push_notifications = false;
    }

    function changeTheme(event: Event) {
      if (event.target instanceof HTMLSelectElement) {
        setTheme(event.target.value);
      }
    }

</script>

<div class="settings">

    <!-- Top Right Corner - Close Button -->
    <button id='btn-close' aria-label="Close Client Settings" onclick={onClose}>
        <span class="icon" aria-hidden="true">{@html icon_x}</span>
    </button>

    <h1>Client Settings</h1>

    <hr/>

    <h2>Notifications:</h2>

    {#if browser_notifications == false}
        <button id="btn-notifications-on">Turn On Notifications</button>
    {:else}
        <button id="btn-notifications-off">Turn Off Notifications</button>
    {/if}
    <small>Brower Notifications: {browser_notifications}</small>

    {#if push_notifications}
        <button id="btn-push-on">Turn On Push Notifications</button>
    {:else}
        <button id="btn-push-off">Turn Off Push Notifications</button>
    {/if}
    <small>Push Notifications: {push_notifications}</small>

    <hr/>

    {#if is_admin}
        <h2>Admin</h2>
        <button id="btn-create-reg-code">Create Registration Code</button>
    {/if}

    <h2>User</h2>
    <label for="theme">Theme</label>
    <select id="theme" value={getTheme()} onchange={changeTheme}>
        <option value="dark">Dark</option>
        <option value="light">Light</option>
    </select>
    <button id='btn-sign-out' onclick={signOut}>Sign Out</button>


</div>
