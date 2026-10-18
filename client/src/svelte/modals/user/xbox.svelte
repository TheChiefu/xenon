<script lang="ts">
  import { LABELS_PLATFORM, ICONS_PLATFORM } from "@/lib/labels";
  import { MSG_SAVED, MSG_ERROR, MSG_UNLINK_XBOX, refreshedText } from "@/lib/labels";
  import * as events from "@/lib/events";
  import { Platform } from "@/bindings/shared";
  import type { LinkResponse, XboxResponse } from "@/bindings/services/xbox";
  import * as xboxAPI from "@/lib/api/xbox";
  import { failureText } from "@/lib/utils";
  import * as profile from "@/lib/users/profile.svelte";
  import icon_link from "@/assets/icons/link.svg?raw";
  import icon_link_broken from "@/assets/icons/link_broken.svg?raw";
  import icon_refresh from "@/assets/icons/refresh.svg?raw";

  let xbox_account: XboxResponse | null = $state(null);
  let xbox_notify = $state("");

  // Loads linked Xbox account when tab opens
  $effect(() => {
    loadXboxAccount();
  });

  // Fetch xbox account settings from the server
  async function loadXboxAccount(): Promise<void> {
    xbox_notify = "";

    try {
      xbox_account = await xboxAPI.getAccount();
      await profile.load();
    } catch (failure) {
      xbox_notify = `${MSG_ERROR}: ${failureText(failure)}`;
    }
  }

  // Reloads Xbox account to show one linked in sign-in tab
  async function refreshXboxAccount(): Promise<void> {
    await loadXboxAccount();

    if (xbox_notify === "") {
      xbox_notify = refreshedText();
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

      const link: LinkResponse = await xboxAPI.startLink();
      sign_in.location.href = link.authorize_url;
    } catch (failure) {
      sign_in?.close();
      xbox_notify = `${MSG_ERROR}: ${failureText(failure)}`;
    }
  }

  // Delete xbox account entry in server
  async function unlinkXboxAccount(): Promise<void> {
    xbox_notify = "";

    const accepted = await events.confirm(MSG_UNLINK_XBOX);

    if (!accepted) {
      return;
    }

    try {
      // Unlink on server and remove entry from current settings page
      await xboxAPI.unlink();
      xbox_account = null;
      await profile.load();
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
      await xboxAPI.updateAccount({ honor_cloaked: xbox_account.honor_cloaked });
      xbox_notify = `(${MSG_SAVED})`;
    } catch (failure) {
      xbox_notify = `(${MSG_ERROR}: ${failureText(failure)})`;
    }
  }
</script>

<div class="section platform">
    <div class="platform-row">
      <span class="icon" title={LABELS_PLATFORM[Platform.xbox]}>{@html ICONS_PLATFORM[Platform.xbox]}</span>
      <p class="title">Xbox</p>
      <p class="message">{xbox_notify || "​"}</p>
    </div>

    <div class="platform-row">

      <div class="platform-text">
        <p>Gamertag:</p>
        <p class="handle">{xbox_account?.gamertag ?? "Not Linked"}</p>
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
      <hr/>
      <p class="title">Account Settings:</p>
      <div class="platform-row" title="Hide Activity While Appearing Offline">
        <label for="xbox-honor-cloaked">Honor Offline Status</label>
        <input id="xbox-honor-cloaked" type="checkbox" bind:checked={xbox_account.honor_cloaked}/>
      </div>

      <button onclick={saveSettings}>Save Changes</button>
    {/if}
</div>

<style>
  label {
    pointer-events: none;
  }
</style>
