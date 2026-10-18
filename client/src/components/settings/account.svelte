<script lang="ts">
  import { MSG_SAVED, MSG_ERROR, MSG_ANONYMIZE, MSG_DELETE_ACCOUNT } from "@/lib/labels";
  import * as usersAPI from "@/lib/api/users";
  import * as events from "@/lib/events";
  import * as session from "@/lib/session.svelte";
  import { failureText } from "@/lib/utils";

  // User Credentials
  let creds_password_curr = $state("");
  let creds_password_new = $state("");
  let creds_revoke_sessions = $state(false);
  let creds_notify = $state("");

  // Account Deletion
  let delete_anonymize = $state(false);
  let delete_history = $state(false);
  let delete_notify = $state("");

  async function savePassword() {
    creds_notify = "";

    try {
      await usersAPI.changePassword(creds_password_curr, creds_password_new, creds_revoke_sessions);
      creds_password_curr = "";
      creds_password_new = "";
      creds_revoke_sessions = false;
      creds_notify = MSG_SAVED;
    } catch (failure) {
      creds_notify = `${MSG_ERROR}: ${failureText(failure)}`;
    }
  }

  async function deleteAccount() {
    const accepted = await events.confirm(MSG_DELETE_ACCOUNT);

    if (!accepted) {
      return;
    }

    try {
      await usersAPI.deleteMe(delete_anonymize, delete_history);
      session.removeLoggedInAccount();
    } catch (failure) {
      delete_notify = `${MSG_ERROR}: ${failureText(failure)}`;
    }
  }
</script>

<div class="section fields">
    <div class="grid-title">
        <h3>Credentials</h3>
    </div>
    <p class="message">{creds_notify}</p>

    <label for="current-password">Current Password</label>
    <input id="current-password" name="current-password" type="password" autocomplete="current-password" bind:value={creds_password_curr}/>

    <label for="new-password">New Password</label>
    <input id="new-password" name="new-password" type="password" autocomplete="new-password" bind:value={creds_password_new}/>

    <label for="revoke-sessions">Sign Out Other Sessions</label>
    <input id="revoke-sessions" type="checkbox" bind:checked={creds_revoke_sessions}/>
    <button class="full-row" onclick={savePassword}>Save Changes</button>
</div>

<div class="section fields">
    <div class="grid-title">
        <h3 style="color: var(--danger);">Account Deletion</h3>
    </div>
    <p class="message">{delete_notify}</p>

    <label for="delete-username" title={MSG_ANONYMIZE}>Anonymize</label>
    <input id="delete-username" type="checkbox" title={MSG_ANONYMIZE} bind:checked={delete_anonymize}/>
    <label for="delete-messages">Delete All Messages</label>
    <input id="delete-messages" type="checkbox" bind:checked={delete_history}/>
    <button class="full-row is-danger" onclick={deleteAccount}>Delete Account</button>
</div>
