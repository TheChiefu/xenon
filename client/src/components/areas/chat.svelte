<script lang="ts">
  import type { MessageResponse } from "@/bindings/routes/messages";
  import Composer from "@/components/composer.svelte";
  import Message from "@/components/message.svelte";
  import * as messages from "@/lib/messages.svelte";
  import * as room from "@/lib/rooms/data.svelte";
  import { getSelectedRoom } from "@/lib/session.svelte";
  import * as users from "@/lib/users.svelte";
  import { failureText } from "@/lib/utils";

  // Pixels from the bottom still counted as at the bottom, since scrollTop can be fractional
  const BOTTOM_MARGIN = 4;

  const roomId: string | null = $derived(getSelectedRoom());
  const roomName: string = $derived(roomId === null ? "" : room.get(roomId)?.name ?? "");
  const roomMessages: MessageResponse[] = $derived(roomId === null ? [] : messages.get(roomId));
  const authorIds: string[] = $derived(uniqueAuthors(roomMessages));

  let list: HTMLDivElement | undefined = $state();
  let failure: string | null = $state(null);

  // Scroll position of each room left this session, keyed by room ID
  const scrollPositions: Map<string, number> = new Map();
  let shownRoom: string | null = null;
  let restorePending = false;
  let atBottom = true;

  // Loads the selected room's newest page
  $effect(() => {
    if (roomId === null) {
      return;
    }

    load(roomId);
  });

  // Fetches summaries of authors not already stored
  $effect(() => {
    users.load(authorIds);
  });

  // Downloads each author's avatar
  $effect(() => {
    for (const id of authorIds) {
      users.loadProfileFiles(id);
    }
  });

  // Before the list changes rooms, stores the scroll position of the room being left
  $effect.pre(() => {
    const nextRoom = roomId;

    if (shownRoom !== null && list !== undefined) {
      scrollPositions.set(shownRoom, list.scrollTop);
    }

    shownRoom = nextRoom;
    restorePending = true;
  });

  // After the list changes, restores a switched room's position, or follows new messages when at the bottom
  $effect(() => {
    if (list === undefined || roomMessages.length === 0) {
      return;
    }

    if (restorePending) {
      restorePending = false;
      list.scrollTop = scrollPositions.get(roomId ?? "") ?? list.scrollHeight;
      return;
    }

    if (atBottom) {
      list.scrollTop = list.scrollHeight;
    }
  });

  async function load(id: string): Promise<void> {
    try {
      await messages.loadLatest(id);
      failure = null;
    } catch (error) {
      failure = failureText(error);
    }
  }

  // Records whether the list is scrolled to the bottom
  function onScroll(): void {
    if (list === undefined) {
      return;
    }

    atBottom = list.scrollHeight - list.scrollTop - list.clientHeight < BOTTOM_MARGIN;
  }

  // Each author ID once
  function uniqueAuthors(fetched: MessageResponse[]): string[] {
    const ids: Set<string> = new Set();
    for (const message of fetched) {
      ids.add(message.author_id);
    }

    return [...ids];
  }
</script>

<style>

  .chat {
    display: flex;
    flex-direction: column;
    flex: 1;
    overflow: hidden;
    min-width: 50%;
  }

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

  .messages {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    overflow-y: auto;
  }

  .composer {
    flex: none;
    display: flex;
    padding: 0.5rem;
    gap: 0.5rem;
    border-top: 1px solid var(--border);
  }

</style>

<div class="chat">

  <div class="header">
    <h3>{roomName}</h3>
  </div>

  <div class="messages" bind:this={list} onscroll={onScroll}>
    {#if failure !== null}
      <p>{failure}</p>
    {/if}

    {#each roomMessages as message (message.id)}
      <Message
        {message}
        author={users.getDisplayName(message.author_id)}
        avatar={users.getAvatar(message.author_id)?.url ?? ""}
      />
    {/each}
  </div>

  {#if roomId !== null}
    <div class="composer">
      <Composer {roomId}/>
    </div>
  {/if}
</div>
