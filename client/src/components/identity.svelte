<script lang="ts">
  import { getActiveLogin, signOut } from "@/lib/session.svelte";
  import { getProfile } from "@/lib/profile.svelte";
  import { getStatus, setStatus } from "@/lib/status.svelte";
  import { LABELS_STATUS } from "@/lib/labels";
  import { Status } from "@/bindings/shared";
  import { getMicEnabled, setMicEnabled, getSoundEnabled, setSoundEnabled } from "@/lib/av.svelte";
  import { confirmDialog } from "@/lib/confirm.svelte";
  import { dialogFly } from "@/lib/transitions";
  import Settings from "@/views/settings.svelte";

  import icon_refresh from "@/assets/icons/refresh.svg?raw";
  import icon_gear from "@/assets/icons/gear.svg?raw";
  import icon_headphones from "@/assets/icons/headphones.svg?raw";
  import icon_headphones_muted from "@/assets/icons/headphones-muted.svg?raw";
  import icon_mic from "@/assets/icons/mic.svg?raw";
  import icon_mic_muted from "@/assets/icons/mic-muted.svg?raw";

  let menuOpen = $state(false);
  let settingsOpen = $state(false);

  // Messages
  const msg_sign_out = "Are you sure you want to sign out?";

  // Closes the menu and opens the settings
  function openSettings() {
    menuOpen = false;
    settingsOpen = true;
  }

  async function doSignout() {
    const result = await confirmDialog(msg_sign_out);
    if (result) {
      signOut();
    }
  }

  function onStatusChange(event: Event) {
    if (!(event.target instanceof HTMLSelectElement)) return;

    setStatus(event.target.value as Status);
  }
</script>

<style>
  .card {
    --point: 35px;

    display: flex;
    width: calc(6rem + var(--point));
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
    height: 100%;
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

<div class="identity">
  <div class="frame status-{getStatus()}">
    <div class="card">
      <button
        class="user"
        aria-expanded={menuOpen}
        onclick={() => menuOpen = !menuOpen}
      >
        <span class="text">
          <span class="user-name">{getProfile()?.display_name ?? ""}</span>
          <span class="user-handle">@{getActiveLogin()?.username}</span>
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
        <button
          class="icon-btn"
          class:is-muted={!getSoundEnabled()}
          aria-label="Toggle sound"
          title="Toggle Sound"
          onclick={() => setSoundEnabled(!getSoundEnabled())}
        >
          <span class="icon" aria-hidden="true">{@html getSoundEnabled() ? icon_headphones : icon_headphones_muted}</span>
        </button>
        <button
          class="icon-btn"
          class:is-muted={!getMicEnabled()}
          aria-label="Toggle microphone"
          title="Toggle Microphone"
          onclick={() => setMicEnabled(!getMicEnabled())}
        >
          <span class="icon" aria-hidden="true">{@html getMicEnabled() ? icon_mic : icon_mic_muted}</span>
        </button>
      </div>

      <select
        id="status"
        title="How you are shown to others when using the client"
        value={getStatus()}
        onchange={onStatusChange}
      >
        {#each Object.values(Status) as value}
          <option {value}>{LABELS_STATUS[value]}</option>
        {/each}
      </select>

      <hr/>

      <button>Switch Account</button>
      <button class="is-danger" onclick={doSignout}>Sign Out</button>
    </div>
    </div>
  {/if}
</div>


{#if settingsOpen}
  <Settings onClose={() => settingsOpen = false}/>
{/if}