<script lang="ts">
  import type { UserProfileResponse, UserSummaryResponse } from "@/bindings/routes/users";
  import { GlobalRole } from "@/bindings/shared";
  import { LABELS_PLATFORM, LABELS_ROLE, MSG_DELETE_USER, MSG_ERROR, MSG_PROMOTE_ADMIN, MSG_SERVER_BAN } from "@/lib/labels";
  import * as usersAPI from "@/lib/api/users";
  import * as events from "@/lib/events";
  import { UserSearch } from "@/lib/users/search.svelte";
  import { failureText } from "@/lib/utils";
  import icon_search from "@/assets/icons/search.svg?raw";
  import icon_user_ban from "@/assets/icons/user-ban.svg?raw";
  import icon_user from "@/assets/icons/user.svg?raw";

  const user_search = new UserSearch();
  let username = $state("");
  let selected = $state<UserSummaryResponse | null>(null);
  let profile = $state<UserProfileResponse | null>(null);
  let delete_history = $state(false);
  let notify = $state("");

  const selected_is_admin = $derived(selected?.global_role === GlobalRole.admin);

  // SEARCH //

  async function search(event: SubmitEvent) {
    event.preventDefault();

    let text: string | undefined = username.trim();
    if (text === "") {
      text = undefined;
    }

    await user_search.search(text);
    selected = null;
    profile = null;
  }

  // SELECTION //

  async function select(user: UserSummaryResponse) {
    selected = user;
    profile = null;
    delete_history = false;
    notify = "";

    try {
      const loaded = await usersAPI.get(user.id);

      // Another user may have been selected while this one loaded
      if (selected?.id === user.id) {
        profile = loaded;
      }
    } catch (failure) {
      notify = `${MSG_ERROR}: ${failureText(failure)}`;
    }
  }

  // ACTIONS //

  async function toggleBan() {
    if (profile === null) return;

    const target = profile;

    if (!target.banned) {
      const accepted = await events.confirm(`${MSG_SERVER_BAN} (@${target.username})`);
      if (!accepted) return;
    }

    notify = "";

    try {
      if (target.banned) {
        await usersAPI.unban(target.id);
      } else {
        await usersAPI.ban(target.id);
      }
      target.banned = !target.banned;
    } catch (failure) {
      notify = `${MSG_ERROR}: ${failureText(failure)}`;
    }
  }

  async function deleteUser() {
    if (selected === null) return;

    const target = selected;

    const accepted = await events.confirm(`${MSG_DELETE_USER} (@${target.username})`);
    if (!accepted) return;

    notify = "";

    try {
      await usersAPI.deleteUser(target.id, { delete_history });
      await user_search.refresh();
      selected = null;
      profile = null;
    } catch (failure) {
      notify = `${MSG_ERROR}: ${failureText(failure)}`;
    }
  }

  async function changeRole(event: Event) {
    if (selected === null) return;
    if (!(event.currentTarget instanceof HTMLSelectElement)) return;

    const target = selected;
    const select = event.currentTarget;
    const role = select.value as GlobalRole;

    if (role === GlobalRole.admin) {
      const accepted = await events.confirm(`${MSG_PROMOTE_ADMIN} (@${target.username})`);
      if (!accepted) {
        select.value = target.global_role;
        return;
      }
    }

    notify = "";

    try {
      await usersAPI.setRole(target.id, { role });
      target.global_role = role;
    } catch (failure) {
      select.value = target.global_role;
      notify = `${MSG_ERROR}: ${failureText(failure)}`;
    }
  }
</script>

<style>
  .users {
    height: 100%;
    min-height: 0;
    display: grid;
    grid-template-columns: minmax(9rem, 14rem) 1fr;
    grid-template-rows: auto 1fr;
    grid-template-areas:
      "search actions"
      "list details";
    gap: var(--row-gap) var(--column-gap);
  }

  @media (max-width: 40rem) {
    .users {
      grid-template-columns: 1fr;
      grid-template-rows: auto minmax(8rem, 1fr) auto auto;
      grid-template-areas:
        "search"
        "list"
        "actions"
        "details";
    }
  }

  .inactive {
    opacity: 0.5;
  }

  .search {
    grid-area: search;
    display: flex;
    gap: var(--inline-gap);
  }

  .actions {
    grid-area: actions;
  }

  .list {
    grid-area: list;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }

  .more {
    margin-top: auto;
    justify-content: center;
  }

  .details {
    grid-area: details;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }

  .description {
    white-space: pre-wrap;
  }

  .footer {
    margin-top: auto;
    display: flex;
    align-items: center;
    gap: var(--inline-gap);
  }

  .error {
    flex: 1;
    color: var(--text-dim);
    font-style: italic;
  }

  .check {
    display: flex;
    align-items: center;
    gap: var(--inline-gap);
  }
</style>

<div class="users">
  <form class="search" onsubmit={search}>
    <input class="is-compact" type="search" placeholder="Username" bind:value={username}/>
    <button
      class="glyph-btn"
      type="submit"
      aria-label="Search"
      title="Search"
      disabled={user_search.pending}
    >
      {@html icon_search}
    </button>
  </form>

  <div class="actions grid-title" class:inactive={selected === null} inert={selected === null}>
    <h4>{selected?.display_name ?? "User"}</h4>

    {#if profile?.banned}
      <button
        class="glyph-btn"
        aria-label="Unban"
        title="Unban"
        onclick={toggleBan}
        disabled={selected_is_admin}
      >
        {@html icon_user}
      </button>
    {:else}
      <button
        class="glyph-btn"
        aria-label="Ban"
        title="Ban"
        onclick={toggleBan}
        disabled={profile === null || selected_is_admin}
      >
        {@html icon_user_ban}
      </button>
    {/if}
  </div>

  <div class="list grid table is-compact">
    {#each user_search.users as user (user.id)}
      <button class:is-active={selected?.id === user.id} onclick={() => select(user)}>
        {user.username}
      </button>
    {/each}

    {#if !user_search.final_page}
      <button class="more" onclick={() => user_search.loadMore()} disabled={user_search.pending}>
        Load More
      </button>
    {/if}
  </div>

  <div class="details grid is-compact" class:inactive={selected === null} inert={selected === null}>
    <div class="fields is-compact">
      <span class="grid-heading">Display Name</span>
      <span>{selected?.display_name ?? ""}</span>

      <span class="grid-heading">Username</span>
      <span>{selected ? `@${selected.username}` : ""}</span>

      <label for="user-role" class="grid-heading">Role</label>
      <span>
        <select
          id="user-role"
          class="is-compact"
          value={selected?.global_role}
          onchange={changeRole}
          disabled={selected === null || selected_is_admin}
        >
          {#each Object.values(GlobalRole) as role (role)}
            <option value={role}>{LABELS_ROLE[role]}</option>
          {/each}
        </select>
      </span>

      <span class="grid-heading">Created</span>
      <span>{profile ? new Date(profile.created_at).toLocaleDateString() : ""}</span>

      <span class="grid-heading">Linked Accounts</span>
      <span>
        {#if profile}
          {#each profile.linked_accounts as link}
            <div>{LABELS_PLATFORM[link.platform]}: {link.handle}</div>
          {:else}
            None
          {/each}
        {/if}
      </span>

      <span class="grid-heading">Banned</span>
      <span>{profile ? (profile.banned ? "Yes" : "No") : ""}</span>

      <span class="grid-heading">Description</span>
      <span class="description">{profile?.description || "None"}</span>
    </div>

    <div class="footer">
      <span class="error">
        {#if notify}
          {notify}
        {:else if user_search.error}
          {MSG_ERROR}: {user_search.error}
        {/if}
      </span>

      <label class="check">
        <input type="checkbox" bind:checked={delete_history}/>
        Delete messages
      </label>

      <button class="is-danger is-compact" onclick={deleteUser} disabled={selected === null}>
        Delete
      </button>
    </div>
  </div>
</div>
