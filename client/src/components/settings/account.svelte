<script lang="ts">
  import { LABELS_PLATFORM, ICONS_PLATFORM, MSG_SAVED, MSG_REFRESHED, MSG_ERROR, MSG_ANONYMIZE, MSG_DELETE_ACCOUNT } from "@/lib/labels";
  import { Platform } from "@/bindings/shared";
  import type { LinkResponse } from "@/bindings/services/xbox";
  import { changePassword, deleteMe } from "@/lib/api/users";
  import { startXboxLink, unlinkXbox } from "@/lib/api/xbox";
  import { confirmDialog } from "@/lib/confirm.svelte";
  import { getProfile, loadProfile } from "@/lib/profile.svelte";
  import { removeLoggedInAccount } from "@/lib/session.svelte";
  import icon_link from "@/assets/icons/link.svg?raw";
  import icon_link_broken from "@/assets/icons/link_broken.svg?raw";
  import icon_refresh from "@/assets/icons/refresh.svg?raw";

  // User Credentials
  let creds_password_curr = $state("");
  let creds_password_new = $state("");
  let creds_revoke_sessions = $state(false);
  let creds_notify = $state("");

  // Linked Accounts
  const account_gamertag: string | null = $derived(getProfile()?.gamertag ?? null);
  let account_linked_steam = $state(false);
  let account_steam_user = $state("");
  let links_notify = $state("");

  // Account Deletion
  let delete_anonymize = $state(false);
  let delete_history = $state(false);
  let delete_notify = $state("");

  async function savePassword() {
    creds_notify = "";

    try {
      await changePassword(creds_password_curr, creds_password_new, creds_revoke_sessions);
      creds_password_curr = "";
      creds_password_new = "";
      creds_revoke_sessions = false;
      creds_notify = MSG_SAVED;
    } catch (failure) {
      creds_notify = MSG_ERROR + (failure instanceof Error ? failure.message : String(failure));
    }
  }

  // Opens Microsoft sign-in in a new tab
  async function linkXboxAccount(): Promise<void> {
    links_notify = "";
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
      links_notify = MSG_ERROR + (failure instanceof Error ? failure.message : String(failure));
    }
  }

  async function unlinkXboxAccount(): Promise<void> {
    links_notify = "";

    try {
      await unlinkXbox();
      await loadProfile();
    } catch (failure) {
      links_notify = MSG_ERROR + (failure instanceof Error ? failure.message : String(failure));
    }
  }

  // Reloads profile to show accounts linked in sign-in tab
  async function refreshLinks(): Promise<void> {
    links_notify = "";

    try {
      await loadProfile();
      const now = new Date();
      links_notify =  `${MSG_REFRESHED} (${now.toLocaleTimeString('en-US', { hour12: true })})`;
    } catch (failure) {
      links_notify = MSG_ERROR + (failure instanceof Error ? failure.message : String(failure));
    }
  }

  async function deleteAccount() {
    const accepted = await confirmDialog(MSG_DELETE_ACCOUNT);

    if (!accepted) {
      return;
    }

    try {
      await deleteMe(delete_anonymize, delete_history);
      removeLoggedInAccount();
    } catch (failure) {
      delete_notify = MSG_ERROR + (failure instanceof Error ? failure.message : String(failure));
    }
  }
</script>

<style>
  .links {
    grid-template-columns: auto 1fr auto;
  }

  .handle {
    font-style: italic;
    color: var(--text-dim);
    cursor: default;
    user-select: none;
  }
</style>

<div class="section rows">
    <h3>Credentials</h3>
    <p class="message">{creds_notify}</p>

    <p>Current Password</p>
    <input name="current-password" type="password" autocomplete="current-password" bind:value={creds_password_curr}/>

    <p>New Password</p>
    <input name="new-password" type="password" autocomplete="new-password" bind:value={creds_password_new}/>

    <label for="revoke-sessions">Sign Out Other Sessions</label>
    <input id="revoke-sessions" type="checkbox" bind:checked={creds_revoke_sessions}/>
    <button class="full-row" onclick={savePassword}>Save Changes</button>
</div>

<div class="section rows links">
    <h3>Linked Accounts</h3>
    <p class="message">{links_notify}</p>
    <button class="icon-btn" aria-label="Refresh Linked Accounts" onclick={refreshLinks}>
      <span class="icon" aria-hidden="true">{@html icon_refresh}</span>
    </button>

    <span class="icon" title={LABELS_PLATFORM[Platform.xbox]}>{@html ICONS_PLATFORM[Platform.xbox]}</span>
    <p class="handle">{account_gamertag ?? "Not Linked"}</p>
    {#if account_gamertag === null}
      <button class="icon-btn" aria-label="Link Xbox" onclick={linkXboxAccount}>
        <span class="icon" aria-hidden="true">{@html icon_link}</span>
      </button>
    {:else}
      <button class="icon-btn is-danger" aria-label="Unlink Xbox" onclick={unlinkXboxAccount}>
        <span class="icon" aria-hidden="true">{@html icon_link_broken}</span>
      </button>
    {/if}

    <span class="icon" title={LABELS_PLATFORM[Platform.steam]}>{@html ICONS_PLATFORM[Platform.steam]}</span>
    <p class="handle">{account_linked_steam ? account_steam_user : "Not Linked"}</p>
    <button
      class="icon-btn {account_linked_steam ? 'is-danger' : ''}"
      aria-label={account_linked_steam ? "Unlink Steam" : "Link Steam"}
      disabled
    >
      <span class="icon" aria-hidden="true">{@html account_linked_steam ? icon_link_broken : icon_link}</span>
    </button>

</div>

<div class="section rows">
    <h3 style="color: var(--danger);">Account Deletion</h3>
    <p class="message">{delete_notify}</p>

    <label for="delete-username" title={MSG_ANONYMIZE}>Anonymize</label>
    <input id="delete-username" type="checkbox" title={MSG_ANONYMIZE} bind:checked={delete_anonymize}/>
    <label for="delete-messages">Delete All Messages</label>
    <input id="delete-messages" type="checkbox" bind:checked={delete_history}/>
    <button class="full-row is-danger" onclick={deleteAccount}>Delete Account</button>
</div>
