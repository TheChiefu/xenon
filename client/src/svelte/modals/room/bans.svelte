<script lang="ts">
  import { MSG_ERROR } from "@/lib/labels";
  import * as settings from "@/lib/rooms/settings.svelte";
  import * as users from "@/lib/users/data.svelte";
  import icon_x from "@/assets/icons/x.svg?raw";

  settings.loadBans();
</script>

<style>
  .bans {
    display: grid;
    grid-template-columns: 1fr 2fr auto auto;
  }

  .notify {
    margin-top: var(--section-gap);
    color: var(--text-dim);
    font-style: italic;
  }
</style>

<div class="fill-column">
  <div class="grid-title">
    <h3>Bans</h3>
  </div>

  <div class="bans grid table">
    <span class="grid-heading">User</span>
    <span class="grid-heading">Reason</span>
    <span class="grid-heading">Expires</span>
    <span class="grid-heading"></span>

    {#each settings.getBans() as entry (entry.user_id)}
      <span>@{users.get(entry.user_id)?.username ?? ""}</span>
      <span>{entry.reason ?? ""}</span>
      <span>{entry.expires_at === null ? "Never" : new Date(entry.expires_at).toLocaleDateString()}</span>
      <span>
        <button class="icon-btn" aria-label="Unban user" title="Unban User" onclick={() => settings.unban(entry.user_id)} disabled={settings.isPending()}>
          <span class="icon" aria-hidden="true">{@html icon_x}</span>
        </button>
      </span>
    {/each}
  </div>

  {#if settings.getError()}
    <span class="notify">{MSG_ERROR}: {settings.getError()}</span>
  {/if}
</div>
