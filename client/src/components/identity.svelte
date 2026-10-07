<script lang="ts">
  import { getActiveLogin, signOut } from "@/lib/session.svelte";
  import { getProfile } from "@/lib/profile.svelte";
  import { getStatus, setStatus } from "@/lib/status.svelte";
  import { LABELS_STATUS } from "@/lib/labels";
  import { Status } from "@/bindings/types";
  import { getMicEnabled, setMicEnabled, getSoundEnabled, setSoundEnabled } from "@/lib/av.svelte";
  import { confirmDialog } from "@/lib/confirm.svelte";
  import { fetchFileBlob, type FetchedFile } from "@/lib/utils";
  import { dialogFly } from "@/lib/transitions";
  import Settings from "@/views/settings.svelte";

  import icon_refresh from "@/assets/icons/refresh.svg?raw";
  import icon_gear from "@/assets/icons/gear.svg?raw";
  import icon_headphones from "@/assets/icons/headphones.svg?raw";
  import icon_headphones_muted from "@/assets/icons/headphones-muted.svg?raw";
  import icon_mic from "@/assets/icons/mic.svg?raw";
  import icon_mic_muted from "@/assets/icons/mic-muted.svg?raw";

  let avatar = $state<FetchedFile | null>(null);
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

  $effect(() => {
    const fileId = getProfile()?.avatar_file_id;
    if (fileId == null) {
      avatar = null;
      return;
    }

    fetchFileBlob(fileId).then((file) => { avatar = file; });

    return () => {
      if (avatar) URL.revokeObjectURL(avatar.url);
    };
  });
</script>

<style>
  .user {
    display: flex;
    align-items: center;
    gap: 0.625rem;
    height: 100%;
    padding: 0 1.125rem;
    border: 0;
    background: var(--component-raised);
    color: var(--text);
    font: inherit;
    text-align: left;
    cursor: pointer;
  }

  .status-online    { box-shadow: inset var(--status-bar-width) 0 0 var(--status-online); }
  .status-busy      { box-shadow: inset var(--status-bar-width) 0 0 var(--status-busy); }
  .status-away      { box-shadow: inset var(--status-bar-width) 0 0 var(--status-away); }
  .status-offline   { box-shadow: inset var(--status-bar-width) 0 0 var(--status-offline); }

  .user-avatar {
    width: 2rem;
    height: 2rem;
    flex: none;
    border-radius: 50%;
    object-fit: cover;
  }

  .user-text {
    display: grid;
    min-width: 0;
  }

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
    border-left: 1px solid var(--border);
    min-width: 6rem;
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
  <button
    class="user status-{getStatus()}"
    aria-expanded={menuOpen}
    onclick={() => menuOpen = !menuOpen}
  >
    {#if avatar?.mime.startsWith("video/")}
      <video class="user-avatar" src={avatar.url} autoplay muted loop playsinline></video>
    {:else if avatar}
      <img class="user-avatar" src={avatar.url} alt=""/>
    {/if}
    <span class="user-text">
      <span class="user-name">{getProfile()?.display_name ?? getActiveLogin()?.username}</span>
      <span class="user-handle">@{getActiveLogin()?.username}</span>
    </span>
  </button>

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