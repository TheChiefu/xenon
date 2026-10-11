<script lang="ts">
  import { LABELS_PLATFORM, ICONS_PLATFORM, MSG_SAVED, MSG_ERROR, MSG_ANONYMIZE, MSG_DELETE_ACCOUNT } from "@/lib/labels";
  import { Platform } from "@/bindings/shared";
  import { changePassword, deleteMe } from "@/lib/api/users";
  import { confirmDialog } from "@/lib/confirm.svelte";
  import { removeLoggedInAccount } from "@/lib/session.svelte";
  import icon_link from "@/assets/icons/link.svg?raw";
  import icon_link_broken from "@/assets/icons/link_broken.svg?raw";

  // User Credentials
  let creds_password_curr = $state("");
  let creds_password_new = $state("");
  let creds_revoke_sessions = $state(false);
  let creds_notify = $state("");

  // Linked Accounts
  let account_linked_xbox = $state(false);
  let account_linked_steam = $state(false);
  let account_gamertag = $state("");
  let account_steam_user = $state("");

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
    <h3 class="full-row">Linked Accounts</h3>

    <span class="icon" title={LABELS_PLATFORM[Platform.xbox]}>{@html ICONS_PLATFORM[Platform.xbox]}</span>
    <p class="handle">{account_linked_xbox ? account_gamertag : "Unlinked"}</p>
    <button
      class="icon-btn {account_linked_xbox ? 'is-danger' : ''}"
      aria-label={account_linked_xbox ? "Unlink Xbox" : "Link Xbox"}
    >
      <span class="icon" aria-hidden="true">{@html account_linked_xbox ? icon_link_broken : icon_link}</span>
    </button>

    <span class="icon" title={LABELS_PLATFORM[Platform.steam]}>{@html ICONS_PLATFORM[Platform.steam]}</span>
    <p class="handle">{account_linked_steam ? account_steam_user : "Unlinked"}</p>
    <button
      class="icon-btn {account_linked_steam ? 'is-danger' : ''}"
      aria-label={account_linked_steam ? "Unlink Steam" : "Link Steam"}
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
