<script lang="ts">
  import type { UserSummaryResponse } from "@/bindings/routes/users";
  import { LABELS_ROLE, MSG_ANONYMIZE, MSG_DELETE_USER, MSG_ERROR } from "@/lib/labels";
  import * as usersAPI from "@/lib/api/users";
  import * as confirm from "@/lib/confirm.svelte";
  import { failureText } from "@/lib/utils";

  let users: UserSummaryResponse[] = $state([]);
  let selected: UserSummaryResponse | null = $state(null);
  let notify = $state("");

  let last_search: string | undefined = undefined;

  async function fetchUsers(username: string | undefined) {
    notify = "";

    try {
      const fetched: UserSummaryResponse[] = [];
      let page: UserSummaryResponse[];

      do {
        const last = fetched.at(-1);

        page = await usersAPI.lookup({
          username,
          after: last === undefined ? undefined : last.id,
        });

        fetched.push(...page);
      } while (page.length > 0);

      users = fetched;
      selected = null;
      last_search = username;
    } catch (failure) {
      notify = `${MSG_ERROR}: ${failureText(failure)}`;
    }
  }

  async function search(event: SubmitEvent) {
    event.preventDefault();
    if (!(event.currentTarget instanceof HTMLFormElement)) return;

    const form = new FormData(event.currentTarget);
    const text = String(form.get("username")).trim();

    await fetchUsers(text === "" ? undefined : text);
  }

  async function deleteUser(event: SubmitEvent) {
    event.preventDefault();
    if (selected === null) return;
    if (!(event.currentTarget instanceof HTMLFormElement)) return;

    const target = selected;
    const form = new FormData(event.currentTarget);
    const anonymize = form.has("anonymize");
    const delete_history = form.has("delete_history");

    const accepted = await confirm.dialog(`${MSG_DELETE_USER} (@${target.username})`);
    if (!accepted) return;

    notify = "";

    try {
      await usersAPI.deleteUser(target.id, { anonymize, delete_history });
      await fetchUsers(last_search);
    } catch (failure) {
      notify = `${MSG_ERROR}: ${failureText(failure)}`;
    }
  }
</script>

<style>
  .search {
    display: flex;
    gap: 0.5rem;
    margin-bottom: 1.25rem;
  }

  .search input {
    flex: 1;
  }

  .layout {
    flex: 1;
    min-height: 0;
    display: grid;
    grid-template-columns: 1fr 2fr;
    gap: 1rem;
  }

  .list {
    display: grid;
    grid-auto-rows: min-content;
    gap: 0.25rem;
  }

  .list button {
    text-align: left;
  }

  .pane {
    grid-template-columns: 8rem 1fr;
    grid-auto-rows: min-content;
  }

  .pane .grid-title {
    grid-column: 1 / -1;
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
    <div class="list grid">
      <span class="grid-heading">Username</span>

      {#each users as user (user.id)}
        <button class:is-active={selected?.id === user.id} onclick={() => selected = user}>
          {user.username}
        </button>
      {/each}
    </div>

    <form class="pane grid fields" onsubmit={deleteUser}>
      <div class="grid-title">
        <h4>{selected === null ? "No User Selected" : selected.display_name}</h4>
      </div>

      {#if selected !== null}
        <span class="grid-heading">Display Name</span>
        <span>{selected.display_name}</span>
        <span class="grid-heading">Username</span>
        <span>@{selected.username}</span>
        <span class="grid-heading">Role</span>
        <span>{LABELS_ROLE[selected.global_role]}</span>

        <div class="grid-title">
          <h4>Delete User</h4>
        </div>

        <label for="delete-anonymize" title={MSG_ANONYMIZE}>Anonymize</label>
        <input id="delete-anonymize" name="anonymize" type="checkbox" title={MSG_ANONYMIZE}/>
        <label for="delete-history">Delete All Messages</label>
        <input id="delete-history" name="delete_history" type="checkbox"/>

        <button type="submit" class="is-danger">Delete</button>
      {/if}
    </form>
  </div>
</div>
