<script lang="ts">
  import type { RegistrationCode } from "@/bindings/shared";
  import { MSG_ERROR, MSG_REVOKE_CODE, refreshedText } from "@/lib/labels";
  import * as authAPI from "@/lib/api/auth";
  import * as events from "@/lib/events";
  import { DAY_MS, failureText } from "@/lib/utils";

  import icon_refresh from "@/assets/icons/refresh.svg?raw";
  import icon_x from "@/assets/icons/x.svg?raw";

  let codes: RegistrationCode[] = $state([]);
  let notify = $state("");
  let create_notify = $state("");

  async function refresh() {
    notify = "";

    try {
      codes = await authAPI.listRegistrationCodes();
      notify = refreshedText();
    } catch (failure) {
      notify = `${MSG_ERROR}: ${failureText(failure)}`;
    }
  }

  async function createCode(event: SubmitEvent) {
    event.preventDefault();
    if (!(event.currentTarget instanceof HTMLFormElement)) return;

    const form = new FormData(event.currentTarget);
    const max_uses = Number(form.get("max_uses"));
    const lifetime = Number(form.get("days")) * DAY_MS;
    create_notify = "";

    try {
      await authAPI.createRegistrationCode({ max_uses, lifetime });
    } catch (failure) {
      create_notify = `${MSG_ERROR}: ${failureText(failure)}`;
      return;
    }

    await refresh();
  }

  async function revokeCode(code: string) {
    if (!await events.confirm(MSG_REVOKE_CODE)) return;

    notify = "";

    try {
      await authAPI.revokeRegistrationCode(code);
      await refresh();
    } catch (failure) {
      notify = `${MSG_ERROR}: ${failureText(failure)}`;
    }
  }
</script>

<style>
  form {
    margin-bottom: var(--section-gap);
  }

  .list-actions {
    display: flex;
    gap: var(--inline-gap);
    align-items: center;
  }

  .notify {
    color: var(--text-dim);
    font-style: italic;
  }

  .codes {
    flex: 1;
    display: grid;
    grid-template-columns: 2fr 1fr 2fr auto;
  }

  .code {
    color: var(--link);
    font-family: monospace;
    text-align: left;
    cursor: pointer;
  }

  .code:hover {
    background: none;
  }

</style>

<div class="fill-column">
  <div class="grid-title">
    <h3>Create Invite</h3>
  </div>

  <form class="fields" onsubmit={createCode}>
    <label for="code-max-uses">Max Uses</label>
    <input id="code-max-uses" name="max_uses" type="number" min="1" value="1" required/>
    <label for="code-days">Lifetime (Days)</label>
    <input id="code-days" name="days" type="number" min="1" value="7" required/>
    <span class="notify">{create_notify}</span>
    <button type="submit">Create</button>
  </form>

  <div class="grid-title">
    <h3>Invites</h3>
    <div class="list-actions">
      <span class="notify">{notify}</span>
      <button class="icon-btn" aria-label="Refresh codes" title="Refresh Codes" onclick={refresh}>
        <span class="icon" aria-hidden="true">{@html icon_refresh}</span>
      </button>
    </div>
  </div>

  <div class="codes grid table">
    <span class="grid-heading">Code</span>
    <span class="grid-heading">Uses</span>
    <span class="grid-heading">Expires</span>
    <span class="grid-heading"></span>

    {#each codes as entry (entry.code)}
      <button class="code inherit" title="Copy Code" onclick={() => navigator.clipboard.writeText(entry.code)}>
        {entry.code}
      </button>
      <span>{entry.uses} / {entry.max_uses}</span>
      <span>{entry.expires_at && new Date(entry.expires_at).toLocaleString()}</span>
      <span>
        <button class="icon-btn" aria-label="Revoke code" title="Revoke Code" onclick={() => revokeCode(entry.code)}>
          <span class="icon" aria-hidden="true">{@html icon_x}</span>
        </button>
      </span>
    {/each}
  </div>

</div>
