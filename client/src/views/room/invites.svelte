<script lang="ts">
  import { MSG_ERROR } from "@/lib/labels";
  import * as settings from "@/lib/rooms/settings.svelte";
  import * as users from "@/lib/users/data.svelte";
  import { UserSearch } from "@/lib/users/search.svelte";
  import icon_x from "@/assets/icons/x.svg?raw";

  const user_search = new UserSearch();

  settings.loadInvites();

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
            <button onclick={() => settings.invite(user.id)} disabled={settings.isPending()}>Invite</button>
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
