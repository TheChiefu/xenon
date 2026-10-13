<script lang="ts">
  import { LABELS_PLATFORM, ICONS_PLATFORM, MSG_SAVED, MSG_REFRESHED, MSG_ERROR, MSG_UNLINK_XBOX } from "@/lib/labels";
  import { confirmDialog } from "@/lib/confirm.svelte";
  import { Platform } from "@/bindings/shared";
  import type { LinkResponse, XboxResponse } from "@/bindings/services/xbox";
  import { getXboxAccount, startXboxLink, unlinkXbox, updateXboxAccount } from "@/lib/api/xbox";
  import { failureText } from "@/lib/utils";
  import icon_link from "@/assets/icons/link.svg?raw";
  import icon_link_broken from "@/assets/icons/link_broken.svg?raw";
  import icon_refresh from "@/assets/icons/refresh.svg?raw";

  let xbox_account: XboxResponse | null = $state(null);
  let xbox_notify = $state("");

  // Loads linked Xbox account when tab opens
  $effect(() => {
    loadXboxAccount();
  });

  async function loadXboxAccount(): Promise<void> {
    xbox_notify = "";

    try {
      xbox_account = await getXboxAccount();
    } catch (failure) {
      xbox_notify = `${MSG_ERROR}: ${failureText(failure)}`;
    }
  }

  // Reloads Xbox account to show one linked in sign-in tab
  async function refreshXboxAccount(): Promise<void> {
    await loadXboxAccount();

    if (xbox_notify === "") {
      const now = new Date();
      xbox_notify = `(${MSG_REFRESHED} ${now.toLocaleTimeString('en-US', { hour12: true })})`;
    }
  }

  // Opens Microsoft sign-in in a new tab
  async function linkXboxAccount(): Promise<void> {
    xbox_notify = "";
    let sign_in: Window | null = null;

    try {
      // Opened before any await, browsers block tabs opened later
      sign_in = window.open("", "_blank");

      if (sign_in === null) {
        throw new Error("Sign-in tab was blocked");
      }

      const link: LinkResponse = await startXboxLink();
      sign_in.location.href = link.authorize_url;
    } catch (failure) {
      sign_in?.close();
      xbox_notify = `${MSG_ERROR}: ${failureText(failure)}`;
    }
  }

  async function unlinkXboxAccount(): Promise<void> {
    xbox_notify = "";

    const accepted = await confirmDialog(MSG_UNLINK_XBOX);

    if (!accepted) {
      return;
    }

    try {
      await unlinkXbox();
      xbox_account = null;
    } catch (failure) {
      xbox_notify = `${MSG_ERROR}: ${failureText(failure)}`;
    }
  }

  async function saveSettings(): Promise<void> {
    xbox_notify = "";

    if (xbox_account === null) {
      return;
    }

    try {
      await updateXboxAccount({ honor_cloaked: xbox_account.honor_cloaked });
      xbox_notify = `(${MSG_SAVED})`;
    } catch (failure) {
      xbox_notify = `(${MSG_ERROR}: ${failureText(failure)})`;
    }
  }
</script>

<style>
  .title {
    font-size: 1.1rem;
    font-weight: 500;
    margin: 0;
  }

  .platform {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .platform-row {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .platform-text {
    display: flex;
    align-items: baseline;
    gap: 0.5rem;
  }

  .platform-text,
  .platform-row label {
    flex: 1;
  }

  .platform-buttons {
    display: flex;
    gap: 0.5rem;
  }

  .handle {
    font-style: italic;
    color: var(--text-dim);
    cursor: default;
    user-select: none;
  }
</style>

<div class="section platform">
    <div class="platform-row">

      <div class="platform-text">
        <p class="title">Xbox:</p>
        <p class="handle">{xbox_account?.gamertag ?? "Not Linked"}</p>
        <p class="message">{xbox_notify || "​"}</p>
      </div>

      <div class="platform-buttons">
        {#if xbox_account === null}
          <button class="icon-btn" aria-label="Link Xbox" onclick={linkXboxAccount}>
            <span class="icon" aria-hidden="true">{@html icon_link}</span>
          </button>
        {:else}
          <button class="icon-btn is-danger" aria-label="Unlink Xbox" onclick={unlinkXboxAccount}>
            <span class="icon" aria-hidden="true">{@html icon_link_broken}</span>
          </button>
        {/if}

        <button class="icon-btn" aria-label="Refresh Xbox Account" onclick={refreshXboxAccount}>
          <span class="icon" aria-hidden="true">{@html icon_refresh}</span>
        </button>
      </div>
    </div>

    {#if xbox_account !== null}
      <p class="title">Account Settings:</p>
      <div class="platform-row">
        <label for="xbox-honor-cloaked">Hide Activity While Appearing Offline</label>
        <input id="xbox-honor-cloaked" type="checkbox" bind:checked={xbox_account.honor_cloaked}/>
      </div>

      <button onclick={saveSettings}>Save Changes</button>
    {/if}
</div>
