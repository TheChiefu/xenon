<script lang="ts">
  import { MSG_ERROR } from "@/lib/labels";
  import * as invites from "@/lib/rooms/invites.svelte";
  import * as users from "@/lib/users/data.svelte";
  import icon_check from "@/assets/icons/check.svg?raw";
  import icon_refresh from "@/assets/icons/refresh.svg?raw";
  import icon_x from "@/assets/icons/x.svg?raw";
</script>

<style>

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

  .list {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    overflow-y: auto;
  }

  .invite {
    display: flex;
    flex: none;
    align-items: center;
    padding: 0.25rem 0.5rem;
    background-color: var(--component);
  }

  .text {
    display: grid;
    flex: 1;
    min-width: 0;
  }

  .room, .sender {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .sender {
    color: var(--text-dim);
    font-size: 0.78rem;
  }

</style>

<div class="header">
  <h3>Inbox: {invites.getInvites().length}</h3>
  <button class="icon-btn" aria-label="Refresh inbox" title="Refresh inbox" onclick={invites.load} disabled={invites.isPending()}>
    <span class="icon" aria-hidden="true">{@html icon_refresh}</span>
  </button>
</div>

{#if invites.getError() !== null}
  <p class="failure">{MSG_ERROR}: {invites.getError()}</p>
{/if}

{#if invites.getInvites().length === 0}
  <p class="empty">No Invites</p>
{/if}

<div class="list">
  {#each invites.getInvites() as entry (entry.room_id)}
    <div class="invite">
      <span class="text">
        <span class="room">{entry.room_name ?? "Unnamed Room"}</span>
        <span class="sender">Invited by {users.getDisplayName(entry.invited_by)}</span>
      </span>

      <button class="icon-btn" aria-label="Accept invite" title="Accept Invite" onclick={() => invites.accept(entry.room_id)} disabled={invites.isPending()}>
        <span class="icon" aria-hidden="true">{@html icon_check}</span>
      </button>
      <button class="icon-btn" aria-label="Decline invite" title="Decline Invite" onclick={() => invites.decline(entry.room_id)} disabled={invites.isPending()}>
        <span class="icon" aria-hidden="true">{@html icon_x}</span>
      </button>
    </div>
  {/each}
</div>
