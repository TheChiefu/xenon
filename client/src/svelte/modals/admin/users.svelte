<script lang="ts">
  import type { UserSummaryResponse } from "@/bindings/routes/users";
  import { GlobalRole } from "@/bindings/shared";
  import { LABELS_ROLE, MSG_DELETE_USER, MSG_ERROR, MSG_PROMOTE_ADMIN } from "@/lib/labels";
  import * as usersAPI from "@/lib/api/users";
  import * as events from "@/lib/events";
  import { UserSearch } from "@/lib/users/search.svelte";
  import { failureText } from "@/lib/utils";

  const user_search = new UserSearch();
  let selected: UserSummaryResponse | null = $state(null);
  let notify = $state("");

  async function search(event: SubmitEvent) {
    event.preventDefault();
    if (!(event.currentTarget instanceof HTMLFormElement)) return;

    const form = new FormData(event.currentTarget);
    const text = String(form.get("username")).trim();

    await user_search.search(text === "" ? undefined : text);
    selected = null;
  }

  async function deleteUser(event: SubmitEvent) {
    event.preventDefault();
    if (selected === null) return;
    if (!(event.currentTarget instanceof HTMLFormElement)) return;

    const target = selected;
    const form = new FormData(event.currentTarget);
    const delete_history = form.has("delete_history");

    const accepted = await events.confirm(`${MSG_DELETE_USER} (@${target.username})`);
    if (!accepted) return;

    notify = "";

    try {
      await usersAPI.deleteUser(target.id, { delete_history });
      await user_search.refresh();
      selected = null;
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
  label {
    pointer-events: none;
  }

  .search {
    display: flex;
    gap: var(--inline-gap);
    margin-bottom: var(--section-gap);
  }

  .search input {
    flex: 1;
  }

  .layout {
    flex: 1;
    min-height: 0;
    display: grid;
    grid-template-columns: 1fr 2fr;
    gap: var(--section-gap);
  }

  @container (width < 30rem) {
    .layout {
      grid-template-columns: 1fr;
      grid-template-rows: 1fr 2fr;
    }
  }

  .fill-column {
    container-type: inline-size;
  }

  .list {
    display: grid;
  }

  .pane {
    grid-template-columns: 8rem 1fr;
    grid-auto-rows: min-content;
  }

  .pane .grid-title {
    grid-column: 1 / -1;
  }

  .empty {
    display: grid;
    place-items: center;
    user-select: none;
  }

  .notify {
    margin-top: var(--section-gap);
    color: var(--text-dim);
    font-style: italic;
  }

  .pane .grid-title:first-child {
    position: sticky;
    top: 0;
    background: var(--component);
  }
</style>

<div class="fill-column">
  <div class="grid-title">
    <h3>Search Users</h3>
  </div>

  <form class="search" onsubmit={search}>
    <input name="username" type="search" placeholder="Username"/>
    <button type="submit">Search</button>
  </form>

  <div class="layout">
    <div class="list grid table">
      <span class="grid-heading">Username</span>

      {#each user_search.users as user (user.id)}
        <button class:is-active={selected?.id === user.id} onclick={() => selected = user}>
          {user.username}
        </button>
      {/each}

      {#if !user_search.final_page}
        <button onclick={() => user_search.loadMore()} disabled={user_search.pending}>Load More</button>
      {/if}
    </div>

    {#if selected === null}
      <div class="grid grid-heading empty">No User Selected</div>
    {:else}
      <form class="pane grid fields" onsubmit={deleteUser}>
        <div class="grid-title">
          <h4>{selected.display_name}</h4>
        </div>

        <span class="grid-heading">Display Name</span>
        <span>{selected.display_name}</span>
        <span class="grid-heading">Username</span>
        <span>@{selected.username}</span>
        <label for="user-role" class="grid-heading">Role</label>
        <select id="user-role" value={selected.global_role} onchange={changeRole} disabled={selected.global_role === GlobalRole.admin}>
          {#each Object.values(GlobalRole) as role (role)}
            <option value={role}>{LABELS_ROLE[role]}</option>
          {/each}
        </select>

        <div class="grid-title">
          <h4>Delete User</h4>
        </div>

        <label for="delete-history">Delete All Messages</label>
        <input id="delete-history" name="delete_history" type="checkbox"/>

        <button type="submit" class="is-danger">Delete</button>
      </form>
    {/if}
  </div>

  {#if user_search.error}
    <span class="notify">{MSG_ERROR}: {user_search.error}</span>
  {/if}

  {#if notify}
    <span class="notify">{notify}</span>
  {/if}
</div>
