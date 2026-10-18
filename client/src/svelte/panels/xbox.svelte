<script lang="ts">
  import type { FriendResponse } from "@/bindings/services/xbox";
  import { failureText } from "@/lib/utils";
  import * as xbox from "@/lib/xbox.svelte";
  import icon_lock from "@/assets/icons/lock.svg?raw";
  import icon_refresh from "@/assets/icons/refresh.svg?raw";
  import icon_users from "@/assets/icons/users.svg?raw";

  let failure: string | null = $state(null);

  async function refreshFriends(): Promise<void> {
    if (!xbox.isLinked()) {
      return;
    }

    try {
      await xbox.loadFriends();
      failure = null;
    } catch (error) {
      failure = failureText(error);
    }
  }

  // Online first, then alphabetical by gamertag
  function compareFriends(a: FriendResponse, b: FriendResponse): number {
    if (a.online !== b.online) {
      return a.online ? -1 : 1;
    }

    return a.gamertag.localeCompare(b.gamertag);
  }

  const ordered: FriendResponse[] = $derived(xbox.getFriends().toSorted(compareFriends));
</script>

<style>

  /* Pane */


  /* Header */

  .header {
    display: flex;
    flex-direction: row;
    flex: none;
    padding: 0.5rem;
    align-items: center;
  }

  .header h3 {
    margin-right: auto;
    font-size: 1rem;
  }

  .failure, .empty {
    flex: none;
    margin: 0;
    padding: 0.5rem;
    font-size: 0.85rem;
  }

  .failure {
    color: var(--danger);
  }

  .empty {
    color: var(--text-dim);
    font-style: italic;
  }

  /* List */

  .list {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    overflow-y: auto;
  }

  /* Rows */

  .friend {
    --avatar-size: 3rem;
    height: var(--avatar-size);
    display: flex;
    flex: none;
    align-items: center;
    background-color: var(--component);
    border: 0.25rem solid transparent;
  }

  .friend:hover {
    border-color: rgba(255,255,255,0.1);
  }

  .picture {
    flex: none;
    width: var(--avatar-size);
    height: var(--avatar-size);
    object-fit: contain;
  }

  .text {
    display: grid;
    flex: 1;
    min-width: 0;
    padding: 0 0.5rem;
  }

  .name, .presence {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .presence {
    color: var(--text-dim);
    font-size: 0.78rem;
  }

  .party {
    flex: none;
    display: flex;
    flex-direction: column;
    align-items: center;
    margin-right: 0.5rem;
  }

  .count {
    color: var(--text-dim);
    font-size: 0.7rem;
    line-height: 1;
    user-select: none;
  }

  .icons {
    display: flex;
  }

  /* Status */
  .online  {
    background-color: color-mix(in srgb, var(--status-online), black 70%);
  }


</style>

<!-- Display Header -->
<div class="header">
  <h3>Xbox Friends: {xbox.getFriends().length}</h3>
  <button class="icon-btn" aria-label="Refresh friends" title="Refresh friends" onclick={refreshFriends}>
    <span class="icon" aria-hidden="true">{@html icon_refresh}</span>
  </button>
</div>

{#if !xbox.isLinked()}
  <p class="empty">No Xbox Account Linked</p>
{:else if failure !== null}
  <p class="failure">Failure: {failure}</p>
{/if}

<!-- Friend List -->
<div class="list">
  {#each ordered as friend (friend.gamertag)}
    <div class="friend" class:online={friend.online} >

      {#if friend.picture_url === null}
        <span class="picture"></span>
      {:else}
        <img class="picture" src={friend.picture_url} alt=""/>
      {/if}

      <span class="text">
        <span class="name">{friend.gamertag}</span>
        <span class="presence">{friend.presence_text}</span>
      </span>

      {#if friend.in_party > 0}
        <span
          class="party"
          aria-label={friend.party_locked ? "Invite Only" : "In Party"}
          title={friend.party_locked ? "Invite Only" : "In Party"}
        >
          <span class="count">{friend.in_party}</span>
          <span class="icons">
            <span class="icon" aria-hidden="true">{@html icon_users}</span>
            {#if friend.party_locked}
              <span class="icon" aria-hidden="true">{@html icon_lock}</span>
            {/if}
          </span>
        </span>
      {/if}
    </div>
  {/each}
</div>
