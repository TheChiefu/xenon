<script lang="ts">
  import type { FriendResponse } from "@/bindings/services/xbox";
  import { HANDLE_WIDTH } from "@/lib/resize.svelte";
  import { failureText } from "@/lib/utils";
  import { getFriends, isXboxLinked, loadFriends, xboxWidth } from "@/lib/xbox.svelte";
  import icon_refresh from "@/assets/icons/refresh.svg?raw";
  import icon_users from "@/assets/icons/users.svg?raw";

  let pane: HTMLDivElement | undefined = $state();
  let failure: string | null = $state(null);

  // Loads friends when pane opens or an account is linked
  $effect(() => {
    refreshFriends();
  });

  async function refreshFriends(): Promise<void> {
    if (!isXboxLinked()) {
      return;
    }

    try {
      await loadFriends();
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

  const ordered: FriendResponse[] = $derived(getFriends().toSorted(compareFriends));
</script>

<style>

  /* Pane */

  .xbox {
    position: relative;
    display: flex;
    flex-direction: column;
    flex: none;
    margin-left: auto;
    border-left: 1px solid var(--border);
    background: rgba(30,30,30, 0.5);
    overflow: hidden;
  }

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
  }

  .picture {
    flex: none;
    width: var(--avatar-size);
    height: var(--avatar-size);
    object-fit: cover;
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
    margin-right: 0.5rem;
  }

  /* Status Bar */

  .status {
    width: var(--status-bar-width);
    flex: none;
    align-self: stretch;
  }

  .online  { background: var(--status-online); }
  .offline { background: var(--status-offline); }

  /* Resize Handle */

  .resize-handle {
    position: absolute;
    top: 0;
    left: calc(var(--handle-width) / -2);
    width: var(--handle-width);
    height: 100%;
    z-index: 1;
    touch-action: none;
    background-color: var(--border);
    cursor: col-resize;
  }

  .resize-handle.dragging {
    background-color: var(--btn-action);
  }

</style>

<!-- Xbox Pane -->
<div
  class="xbox"
  style="width: {xboxWidth.getWidth()}px"
  style:--handle-width="{HANDLE_WIDTH}px"
  bind:this={pane}
>

  <!-- Display Header -->
  <div class="header">
    <h3>Xbox Friends: {getFriends().length}</h3>
    <button class="icon-btn" aria-label="Refresh friends" title="Refresh friends" onclick={refreshFriends}>
      <span class="icon" aria-hidden="true">{@html icon_refresh}</span>
    </button>
  </div>

  {#if !isXboxLinked()}
    <p class="empty">No Xbox Account Linked</p>
  {:else if failure !== null}
    <p class="failure">Failure: {failure}</p>
  {/if}

  <!-- Friend List -->
  <div class="list">
    {#each ordered as friend (friend.gamertag)}
      <div class="friend">
        {#if friend.picture_url === null}
          <span class="picture"></span>
        {:else}
          <img class="picture" src={friend.picture_url} alt=""/>
        {/if}

        <span class="text">
          <span class="name">{friend.gamertag}</span>
          <span class="presence">{friend.presence_text}</span>
        </span>

        {#if friend.in_party}
          <span class="icon party" aria-label="In party" title="In party">{@html icon_users}</span>
        {/if}

        {#if friend.online}
          <span class="status online"></span>
        {:else}
          <span class="status offline"></span>
        {/if}
      </div>
    {/each}
  </div>

  <!-- Vertical Bar for resizing Xbox pane -->
  <div
    class="resize-handle"
    class:dragging={xboxWidth.isResizing()}
    role="separator"
    aria-orientation="vertical"
    aria-label="Resize Xbox pane"
    onpointerdown={(event) => xboxWidth.resizeStart(event)}
    onpointermove={(event) => xboxWidth.resize(event, pane)}
    onpointerup={() => xboxWidth.resizeStop()}
    onlostpointercapture={() => xboxWidth.resizeStop()}
    ondblclick={() => xboxWidth.reset()}
  ></div>
</div>
