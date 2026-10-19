<script lang="ts">
  import { MSG_ERROR } from "@/lib/labels";
  import * as members from "@/lib/rooms/members.svelte";
  import * as settings from "@/lib/rooms/settings.svelte";
  import * as users from "@/lib/users/data.svelte";
  import { UserSearch } from "@/lib/users/search.svelte";
  import { DAY_MS } from "@/lib/utils";
  import icon_x from "@/assets/icons/x.svg?raw";

  const user_search = new UserSearch();

  let expire_days: number | undefined = $state(undefined);

  settings.loadInvites();

  function isMember(user_id: string): boolean {
    const room = settings.getRoom();
    if (room === undefined) return false;

    return members.has(room.id, user_id);
  }

  function isInvited(user_id: string): boolean {
    for (const entry of settings.getInvites()) {
      if (entry.user_id === user_id) return true;
    }

    return false;
  }

  // Milliseconds until a new invite expires, null for never
  function expireDelta(): number | null {
    if (expire_days === undefined) return null;

    return expire_days * DAY_MS;
  }

  function invite(user_id: string): void {
    settings.invite(user_id, expireDelta());
  }

  async function search(event: SubmitEvent) {
    event.preventDefault();
    if (!(event.currentTarget instanceof HTMLFormElement)) return;

    const form = new FormData(event.currentTarget);
    const text = String(form.get("username")).trim();

    await user_search.search(text === "" ? undefined : text);
  }
</script>

<style>
  .wrapper {
    container-type: inline-size;
  }

  .columns {
    flex: 1;
    min-height: 0;
    display: grid;
    grid-template-columns: 1fr 1fr;
    gap: var(--section-gap);
  }

  @container (width < 30rem) {
    .columns {
      grid-template-columns: 1fr;
    }
  }

  .search {
    display: flex;
    gap: var(--inline-gap);
    margin-bottom: var(--section-gap);
  }

  .search input {
    flex: 1;
  }

  .results {
    display: grid;
    grid-template-columns: 1fr auto;
  }

  .invites {
    display: grid;
    grid-template-columns: 1fr auto auto;
  }

  .span {
    grid-column: 1 / -1;
  }

  .notify {
    margin-top: var(--section-gap);
    color: var(--text-dim);
    font-style: italic;
  }
</style>

<div class="wrapper fill-column">
  <div class="columns">
    <div class="fill-column">
      <div class="grid-title">
        <h3>Invite User</h3>
      </div>

      <div class="fields">
        <label for="invite-days">Expires In (Days)</label>
        <input id="invite-days" type="number" min="1" placeholder="Never" bind:value={expire_days}/>
      </div>

      <form class="search" onsubmit={search}>
        <input name="username" type="search" placeholder="Username"/>
        <button type="submit" disabled={user_search.pending}>Search</button>
      </form>

      <div class="results grid table">
        <span class="grid-heading">Username</span>
        <span class="grid-heading"></span>

        {#each user_search.users as user (user.id)}
          <span>@{user.username}</span>
          <span>
            {#if isMember(user.id)}
              Member
            {:else if isInvited(user.id)}
              Invited
            {:else}
              <button onclick={() => invite(user.id)} disabled={settings.isPending()}>Invite</button>
            {/if}
          </span>
        {/each}

        {#if !user_search.final_page}
          <button class="span" onclick={() => user_search.loadMore()} disabled={user_search.pending}>Load More</button>
        {/if}
      </div>
    </div>

    <div class="fill-column">
      <div class="grid-title">
        <h3>Sent Invites</h3>
      </div>

      <div class="invites grid table">
        <span class="grid-heading">User</span>
        <span class="grid-heading">Expires</span>
        <span class="grid-heading"></span>

        {#each settings.getInvites() as entry (entry.user_id)}
          <span>@{users.get(entry.user_id)?.username ?? ""}</span>
          <span>{entry.expires_at === null ? "Never" : new Date(entry.expires_at).toLocaleDateString()}</span>
          <span>
            <button class="icon-btn" aria-label="Revoke invite" title="Revoke Invite" onclick={() => settings.revokeInvite(entry.user_id)} disabled={settings.isPending()}>
              <span class="icon" aria-hidden="true">{@html icon_x}</span>
            </button>
          </span>
        {/each}
      </div>
    </div>
  </div>

  {#if user_search.error}
    <span class="notify">{MSG_ERROR}: {user_search.error}</span>
  {/if}

  {#if settings.getError()}
    <span class="notify">{MSG_ERROR}: {settings.getError()}</span>
  {/if}
</div>
