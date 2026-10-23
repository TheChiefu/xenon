<script lang="ts">
  import { Permission } from "@/bindings/shared";
  import { LABELS_PERMISSION, MSG_BAN_USER, MSG_ERROR } from "@/lib/labels";
  import * as events from "@/lib/events";
  import * as settings from "@/lib/rooms/settings.svelte";
  import * as profile from "@/lib/users/profile.svelte";
  import { UserSearch } from "@/lib/users/search.svelte";
  import icon_search from "@/assets/icons/search.svg?raw";
  import icon_mail from "@/assets/icons/mail.svg?raw";
  import icon_mail_x from "@/assets/icons/mail-x.svg?raw";
  import icon_user_ban from "@/assets/icons/user-ban.svg?raw";
  import icon_user from "@/assets/icons/user.svg?raw";

  const user_search = new UserSearch();
  let username = $state("");
  let checked_permissions = $state<Permission[]>([]);

  const self_permissions = $derived(settings.getPermissions());
  const selected_id = $derived(settings.getSelectedId());
  const access = $derived(settings.getAccess());
  const selected_is_member = $derived(access !== null && access.permissions !== null);
  const selected_is_self = $derived(selected_id !== null && selected_id === profile.get()?.id);

  // Whether the invite button may be pressed for the selected user
  const invite_enabled = $derived.by(() => {
    if (access === null || selected_is_self || !self_permissions.includes(Permission.invite)) {
      return false;
    }

    if (access.invited) {
      return true;
    }

    return !selected_is_member && !access.banned;
  });

  // Whether the ban button may be pressed for the selected user
  const ban_enabled = $derived(
    access !== null && !selected_is_self && self_permissions.includes(Permission.ban)
  );

  // Whether the selected user's permission checkboxes and Save may be used
  const permissions_enabled = $derived(
    selected_is_member && self_permissions.includes(Permission.grant)
  );

  // Reset the checkboxes to the selected user's permissions
  $effect(() => {
    checked_permissions = [...(access?.permissions ?? [])];
  });

  // SEARCH //

  // Searches server-wide by username
  async function search(event: SubmitEvent) {
    event.preventDefault();

    let text: string | undefined = username.trim();
    if (text === "") {
      text = undefined;
    }

    await user_search.search(text);
  }

  // SELECTION //

  // Selects a user, unless they are already selected
  function selectUser(user_id: string) {
    if (selected_id !== user_id) {
      settings.select(user_id);
    }
  }

  // ACTIONS //

  // Invites the selected user, or revokes their invite
  function toggleInvite() {
    if (selected_id === null || access === null) return;

    if (access.invited) {
      settings.revokeInvite(selected_id);
    } else {
      settings.invite(selected_id);
    }
  }

  // Bans the selected user after confirming, or lifts their ban
  async function toggleBan() {
    if (selected_id === null || access === null) return;

    if (access.banned) {
      settings.unban(selected_id);
      return;
    }

    if (!await events.confirm(MSG_BAN_USER)) return;
    settings.ban(selected_id);
  }

  // PERMISSIONS //

  // Saves the checked permissions for the selected user
  function save(event: SubmitEvent) {
    event.preventDefault();
    if (selected_id === null) return;
    settings.savePermissions(selected_id, checked_permissions);
  }
</script>

<style>
  .access {
    height: 100%;
    min-height: 0;
    display: grid;
    grid-template-columns: minmax(9rem, 14rem) 1fr;
    grid-template-rows: auto 1fr;
    grid-template-areas:
      "search actions"
      "list permissions";
    gap: var(--row-gap) var(--column-gap);
  }

  @media (max-width: 40rem) {
    .access {
      grid-template-columns: 1fr;
      grid-template-rows: auto minmax(8rem, 1fr) auto auto;
      grid-template-areas:
        "search"
        "list"
        "actions"
        "permissions";
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

  .actions {
    grid-area: actions;
  }

  .buttons {
    display: flex;
    gap: var(--inline-gap);
  }

  .permissions {
    grid-area: permissions;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }

  .checks {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(11rem, 1fr));
    gap: var(--row-gap) var(--column-gap);
  }

  .check {
    display: flex;
    align-items: center;
    gap: var(--inline-gap);
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
</style>

<div class="access">
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

  <div class="list grid table is-compact">
    {#each user_search.users as user (user.id)}
      <button
        class:is-active={selected_id === user.id}
        onclick={() => selectUser(user.id)}
        disabled={settings.isPending()}
      >
        {user.username}
      </button>
    {/each}

    {#if !user_search.final_page}
      <button class="more" onclick={() => user_search.loadMore()} disabled={user_search.pending}>
        Load More
      </button>
    {/if}
  </div>

  <div class="actions grid-title" class:inactive={selected_id === null} inert={selected_id === null}>
    <h4>Permissions</h4>

    <div class="buttons">
      {#if access?.invited}
        <button
          class="glyph-btn"
          aria-label="Revoke invite"
          title="Revoke invite"
          onclick={toggleInvite}
          disabled={!invite_enabled || settings.isPending()}
        >
          {@html icon_mail_x}
        </button>
      {:else}
        <button
          class="glyph-btn"
          aria-label="Invite"
          title="Invite"
          onclick={toggleInvite}
          disabled={!invite_enabled || settings.isPending()}
        >
          {@html icon_mail}
        </button>
      {/if}

      {#if access?.banned}
        <button
          class="glyph-btn"
          aria-label="Unban"
          title="Unban"
          onclick={toggleBan}
          disabled={!ban_enabled || settings.isPending()}
        >
          {@html icon_user}
        </button>
      {:else}
        <button
          class="glyph-btn"
          aria-label="Ban"
          title="Ban"
          onclick={toggleBan}
          disabled={!ban_enabled || settings.isPending()}
        >
          {@html icon_user_ban}
        </button>
      {/if}
    </div>
  </div>

  <form
    class="permissions grid is-compact"
    class:inactive={selected_id === null}
    inert={selected_id === null}
    onsubmit={save}
  >
    <div class="checks">
      {#each Object.values(Permission) as value}
        <label class="check">
          <input
            type="checkbox"
            {value}
            bind:group={checked_permissions}
            disabled={!permissions_enabled}
          />
          {LABELS_PERMISSION[value]}
        </label>
      {/each}
    </div>

    <div class="footer">
      <span class="error">
        {#if settings.getError()}
          {MSG_ERROR}: {settings.getError()}
        {:else if user_search.error}
          {MSG_ERROR}: {user_search.error}
        {/if}
      </span>

      <button class="is-compact" type="submit" disabled={!permissions_enabled || settings.isPending()}>
        Save
      </button>
    </div>
  </form>
</div>
