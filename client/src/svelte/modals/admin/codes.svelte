<script lang="ts">
  import type { RegistrationCode } from "@/bindings/types/auth";
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
    const length = Number(form.get("length"));
    create_notify = "";

    try {
      await authAPI.createRegistrationCode({ max_uses, lifetime, length });
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
  .invites {
    height: 100%;
    min-height: 0;
    display: grid;
    grid-template-columns: minmax(9rem, 14rem) 1fr;
    grid-template-rows: auto 1fr;
    grid-template-areas:
      "create-title list-title"
      "create list";
    gap: var(--row-gap) var(--column-gap);
  }

  :global(.is-collapsed) .invites {
    grid-template-columns: 1fr;
    grid-template-rows: auto auto auto minmax(8rem, 1fr);
    grid-template-areas:
      "create-title"
      "create"
      "list-title"
      "list";
  }

  .create-title {
    grid-area: create-title;
  }

  .create {
    grid-area: create;
    gap: var(--row-gap);
  }

  .create .fields {
    grid-template-columns: 1fr;
  }

  .list-title {
    grid-area: list-title;
  }

  .codes {
    grid-area: list;
    min-height: 0;
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

  .expires {
    font-size: 0.85rem;
    color: var(--text-dim);
  }

  .uses {
    font-size: 0.75rem;
  }
</style>

<div class="invites">
  <div class="create-title grid-title">
    <h4>Create Invite</h4>
  </div>

  <form class="create grid is-compact column" onsubmit={createCode}>
    <div class="fields is-compact">
      <label for="code-max-uses" class="grid-heading">Max Uses</label>
      <input id="code-max-uses" class="is-compact" name="max_uses" type="number" min="1" value="1" required/>
      <label for="code-days" class="grid-heading">Lifetime (Days)</label>
      <input id="code-days" class="is-compact" name="days" type="number" min="1" value="7" required/>
      <label for="code-length" class="grid-heading">Length</label>
      <input id="code-length" class="is-compact" name="length" type="number" min="4" max="64" value="12" required/>
    </div>

    <span class="notify">{create_notify}</span>
    <button class="is-compact" type="submit">Create</button>
  </form>

  <div class="list-title grid-title">
    <h4>Invites</h4>

    <div class="row">
      <span class="notify">{notify}</span>
      <button class="glyph-btn" aria-label="Refresh codes" title="Refresh Codes" onclick={refresh}>
        {@html icon_refresh}
      </button>
    </div>
  </div>

  <div class="codes grid table is-compact">
    <span class="grid-heading">Code</span>
    <span class="grid-heading">Uses</span>
    <span class="grid-heading">Expires</span>
    <span class="grid-heading"></span>

    {#each codes as entry (entry.code)}
      <button class="code inherit" title="Copy Code" onclick={() => navigator.clipboard.writeText(entry.code)}>
        {entry.code}
      </button>
      <span class="uses">{entry.uses} / {entry.max_uses}</span>
      <span class="expires">{entry.expires_at && new Date(entry.expires_at).toLocaleString()}</span>
      <span>
        <button class="glyph-btn" aria-label="Revoke code" title="Revoke Code" onclick={() => revokeCode(entry.code)}>
          {@html icon_x}
        </button>
      </span>
    {/each}
  </div>
</div>
