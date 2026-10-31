<script lang="ts">
  import { LABELS_PLATFORM, ICONS_PLATFORM } from "@/lib/labels";
  import { MSG_SAVED, MSG_ERROR, MSG_UNLINK_XBOX, MSG_OPEN_SIGN_IN, MSG_SIGN_IN_READY, MSG_REFRESH_AFTER_SIGN_IN, refreshedText } from "@/lib/labels";
  import * as events from "@/lib/events";
  import { Platform } from "@/bindings/types/platforms";
  import type { LinkResponse, XboxResponse } from "@/bindings/services/xbox";
  import * as xboxAPI from "@/lib/api/xbox";
  import { failureText } from "@/lib/utils";
  import * as profile from "@/lib/users/profile.svelte";
  import icon_link from "@/assets/icons/link.svg?raw";
  import icon_link_broken from "@/assets/icons/link_broken.svg?raw";
  import icon_refresh from "@/assets/icons/refresh.svg?raw";
  import icon_enter from "@/assets/icons/enter.svg?raw";

  let xbox_account: XboxResponse | null = $state(null);
  let xbox_notify = $state("");
  let authorize_url: string | null = $state(null);

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

  // Fetches a Microsoft sign-in url for the sign-in link
  async function linkXboxAccount(): Promise<void> {
    xbox_notify = "";
    authorize_url = null;

    try {
      const link: LinkResponse = await xboxAPI.startLink();
      authorize_url = link.authorize_url;
      xbox_notify = MSG_SIGN_IN_READY;
    } catch (failure) {
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
      authorize_url = null;
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
          <button class="icon-btn" aria-label="Link Xbox" title="Link Xbox" onclick={linkXboxAccount}>
            <span class="icon" aria-hidden="true">{@html icon_link}</span>
          </button>
          {#if authorize_url !== null}
            <a class="icon-btn" href={authorize_url} target="_blank" rel="noopener noreferrer" aria-label={MSG_OPEN_SIGN_IN} title={MSG_OPEN_SIGN_IN} onclick={() => xbox_notify = MSG_REFRESH_AFTER_SIGN_IN}>
              <span class="icon" aria-hidden="true">{@html icon_enter}</span>
            </a>
          {/if}
        {:else}
          <button class="icon-btn is-danger" aria-label="Unlink Xbox" title="Unlink Xbox" onclick={unlinkXboxAccount}>
            <span class="icon" aria-hidden="true">{@html icon_link_broken}</span>
          </button>
        {/if}
        <span class="divider-vertical" aria-hidden="true"></span>
        <button class="icon-btn" aria-label="Refresh Xbox Account" title="Refresh Xbox Account" onclick={refreshXboxAccount}>
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
