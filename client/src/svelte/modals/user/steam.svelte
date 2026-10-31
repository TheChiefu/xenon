<script lang="ts">
  import { LABELS_PLATFORM, ICONS_PLATFORM } from "@/lib/labels";
  import { MSG_ERROR, MSG_UNLINK_STEAM, MSG_OPEN_SIGN_IN, MSG_SIGN_IN_READY, MSG_REFRESH_AFTER_SIGN_IN, refreshedText } from "@/lib/labels";
  import * as events from "@/lib/events";
  import { Platform } from "@/bindings/types/platforms";
  import type { LinkResponse } from "@/bindings/services/steam";
  import * as steamAPI from "@/lib/api/steam";
  import { failureText } from "@/lib/utils";
  import * as profile from "@/lib/users/profile.svelte";
  import icon_link from "@/assets/icons/link.svg?raw";
  import icon_link_broken from "@/assets/icons/link_broken.svg?raw";
  import icon_refresh from "@/assets/icons/refresh.svg?raw";
  import icon_enter from "@/assets/icons/enter.svg?raw";

  let steam_notify = $state("");
  let login_url: string | null = $state(null);

  // The caller's linked Steam account, from their profile
  const steam_account = $derived(
    profile.get()?.linked_accounts.find((account) => account.platform === Platform.steam) ?? null
  );

  // Fetch the caller's profile to read the linked Steam account
  async function loadSteamAccount(): Promise<void> {
    steam_notify = "";

    try {
      await profile.load();
    } catch (failure) {
      steam_notify = `${MSG_ERROR}: ${failureText(failure)}`;
    }
  }

  // Reloads the profile to show an account linked in the sign-in tab
  async function refreshSteamAccount(): Promise<void> {
    await loadSteamAccount();

    if (steam_notify === "") {
      steam_notify = refreshedText();
    }
  }

  // Fetches a Steam sign-in url for the sign-in link
  async function linkSteamAccount(): Promise<void> {
    steam_notify = "";
    login_url = null;

    try {
      const link: LinkResponse = await steamAPI.startLink();
      login_url = link.login_url;
      steam_notify = MSG_SIGN_IN_READY;
    } catch (failure) {
      steam_notify = `${MSG_ERROR}: ${failureText(failure)}`;
    }
  }

  // Delete the Steam account entry on the server
  async function unlinkSteamAccount(): Promise<void> {
    steam_notify = "";

    const accepted = await events.confirm(MSG_UNLINK_STEAM);

    if (!accepted) {
      return;
    }

    try {
      // Unlink on server and remove the entry from the current settings page
      await steamAPI.unlink();
      login_url = null;
      await profile.load();
    } catch (failure) {
      steam_notify = `${MSG_ERROR}: ${failureText(failure)}`;
    }
  }
</script>

<div class="section platform">
    <div class="platform-row">
      <span class="icon" title={LABELS_PLATFORM[Platform.steam]}>{@html ICONS_PLATFORM[Platform.steam]}</span>
      <p class="title">Steam</p>
      <p class="message">{steam_notify || "​"}</p>
    </div>

    <div class="platform-row">
      <div class="platform-text">
        <p class="handle">{steam_account?.handle ?? "Not Linked"}</p>
      </div>

      <div class="platform-buttons">
        {#if steam_account === null}
          <button class="icon-btn" aria-label="Link Steam" title="Link Steam" onclick={linkSteamAccount}>
            <span class="icon" aria-hidden="true">{@html icon_link}</span>
          </button>
          {#if login_url !== null}
            <a class="icon-btn" href={login_url} target="_blank" rel="noopener noreferrer" aria-label={MSG_OPEN_SIGN_IN} title={MSG_OPEN_SIGN_IN} onclick={() => steam_notify = MSG_REFRESH_AFTER_SIGN_IN}>
              <span class="icon" aria-hidden="true">{@html icon_enter}</span>
            </a>
          {/if}
        {:else}
          <button class="icon-btn is-danger" aria-label="Unlink Steam" title="Unlink Steam" onclick={unlinkSteamAccount}>
            <span class="icon" aria-hidden="true">{@html icon_link_broken}</span>
          </button>
        {/if}
        <span class="divider-vertical" aria-hidden="true"></span>
        <button class="icon-btn" aria-label="Refresh Steam Account" title="Refresh Steam Account" onclick={refreshSteamAccount}>
          <span class="icon" aria-hidden="true">{@html icon_refresh}</span>
        </button>
      </div>
    </div>
</div>
