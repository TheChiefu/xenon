<script lang="ts">
  import type { MessageResponse } from "@/bindings/routes/messages";
  import { GlobalRole, Permission, Visibility } from "@/bindings/shared";
  import Composer from "./composer.svelte";
  import Message from "./message.svelte";
  import * as messagesAPI from "@/lib/api/messages";
  import * as events from "@/lib/events";
  import { MSG_DELETE_MESSAGE } from "@/lib/labels";
  import * as messages from "@/lib/messages/data.svelte";
  import * as profile from "@/lib/users/profile.svelte";
  import * as room from "@/lib/rooms/data.svelte";
  import * as members from "@/lib/rooms/members.svelte";
  import * as users from "@/lib/users/data.svelte";
  import { failureText } from "@/lib/utils";
  import icon_arrow_up from "@/assets/icons/arrow-up.svg?raw";

  interface Props {
    roomId: string | null;
  }

  let { roomId }: Props = $props();

  
  const BOTTOM_MARGIN = 4; // Pixels from the bottom still counted as at the bottom
  const KEY_SCROLL_POS = "scroll_positions";

  const roomName: string = $derived(roomId === null ? "" : room.get(roomId)?.name ?? "");
  const roomMessages: MessageResponse[] = $derived(roomId === null ? [] : messages.get(roomId));
  const selfId: string = $derived(profile.get()?.id ?? "");
  const canDeleteAny: boolean = $derived(roomId !== null && canDeleteOthers(roomId));

  let list: HTMLDivElement | undefined = $state();
  let content: HTMLDivElement | undefined = $state(); // Contains the messages
  let failure: string | null = $state(null);

  // Scroll position of each room left this session, keyed by room ID
  const scrollPositions: Map<string, number> = new Map(
    JSON.parse(sessionStorage.getItem(KEY_SCROLL_POS) ?? "[]")
  );

  let shownRoom: string | null = null;
  let restorePending = false;
  let atBottom = true;
  let target: number | null = null; // Position being restored

  // Loads the selected room's newest page
  $effect(() => {
    if (roomId === null) {
      return;
    }

    load(roomId);
  });

  // Before the list changes rooms, stores the scroll position of the room being left
  $effect.pre(() => {
    const nextRoom = roomId;
    shownRoom = nextRoom;
    restorePending = true;
    target = null;
  });

  // After the list changes, restores a switched room's position, or follows new messages when at the bottom
  $effect(() => {
    if (list === undefined || roomMessages.length === 0) {
      return;
    }

    if (restorePending) {
      restorePending = false;
      target = scrollPositions.get(roomId ?? "") ?? null;
      list.scrollTop = target ?? list.scrollHeight;
      return;
    }

    if (atBottom) {
      list.scrollTop = list.scrollHeight;
    }
  });

  // Re-applies the restored position as the messages grow, until it is reached
  $effect(() => {
    if (content === undefined) {
      return;
    }

    const observer = new ResizeObserver(() => {
      if (list !== undefined && target !== null) {
        list.scrollTop = target;

        if (Math.abs(list.scrollTop - target) < 1) {
          target = null;
        }
      }
    });

    observer.observe(content);
    return () => observer.disconnect();
  });


  // Methods

  // Stores the shown room's scroll position
  function savePositions(): void {
    if (shownRoom !== null && list !== undefined) {
      scrollPositions.set(shownRoom, list.scrollTop);
      sessionStorage.setItem("scroll_positions", JSON.stringify([...scrollPositions]));
    }
  }

  // Load messages and member within a room
  async function load(id: string): Promise<void> {
    try {
      await messages.loadLatest(id);
      await members.load(id);
      await loadAuthors(id);

      failure = null;
    } catch (error) {
      failure = failureText(error);
    }
  }

  // Load message authors
  // Ones not in user cache are auto skipped when loading
  async function loadAuthors(id: string): Promise<void> {
    const unknown: string[] = [];
    for (const message of messages.get(id)) {
        unknown.push(message.author_id);
    }
    await users.load(unknown);
  }

  // Adds the page before the oldest message shown
  async function showOlder(): Promise<void> {
    if (roomId === null) {
      return;
    }

    try {
      await messages.loadOlder(roomId);
      await loadAuthors(roomId);
      failure = null;
    } catch (error) {
      failure = failureText(error);
    }
  }

  // A message author's display name
  // Marked when they are no longer in the room
  function getAuthorName(authorId: string): string {
    const name = users.getDisplayName(authorId);

    if (roomId !== null && !members.has(roomId, authorId)) {
      return `${name} (Left)`;
    }

    return name;
  }

  // Sends a message's new body, then stores the edit
  async function edit(message: MessageResponse, body: string): Promise<void> {
    try {
      const response = await messagesAPI.edit(message.id, { body });
      messages.edit(message.room_id, message.id, body, response.edited_at);
      failure = null;
    } catch (error) {
      failure = failureText(error);
    }
  }

  // Deletes a message, then removes it from the room
  async function remove(message: MessageResponse, skipConfirm: boolean): Promise<void> {
    if (!skipConfirm && !await events.confirm(MSG_DELETE_MESSAGE)) {
      return;
    }

    try {
      await messagesAPI.remove(message.id);
      messages.remove(message.room_id, message.id);
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

  // Whether the logged in user may delete messages written by others in the room
  function canDeleteOthers(id: string): boolean {

    // Members granted the delete permission in this room
    const permissions = members.getPermissions(id, selfId);
    const canDelete = permissions.includes(Permission.delete_msg);
    if (canDelete) {
      return true;
    }

    // Global admins in a public room
    const isAdmin = profile.get()?.global_role === GlobalRole.admin;
    const isPublic = room.get(id)?.visibility === Visibility.public;

    return isAdmin && isPublic;
  }
</script>

<style>

  .chat {
    display: flex;
    flex-direction: column;
    overflow: hidden;
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
    {#if roomId !== null && messages.hasOlderMessages(roomId)}
      <button class="icon-btn" aria-label="Load More" title="Load More" onclick={showOlder}>
        <span class="icon" aria-hidden="true">{@html icon_arrow_up}</span>
      </button>
    {/if}
  </div>

  <div class="messages" bind:this={list} onscroll={onScroll} onscrollend={savePositions}>
    <div bind:this={content}>
      {#if failure !== null}
        <p>{failure}</p>
      {/if}

      {#each roomMessages as message (message.id)}
        <Message
          {message}
          author={getAuthorName(message.author_id)}
          canEdit={message.author_id === selfId}
          canDelete={canDeleteAny || message.author_id === selfId}
          onEdit={(body) => edit(message, body)}
          onDelete={(skipConfirm) => remove(message, skipConfirm)}
        />
      {/each}
    </div>
  </div>

  {#if roomId !== null}
    <div class="composer">
      <Composer {roomId}/>
    </div>
  {/if}
</div>
