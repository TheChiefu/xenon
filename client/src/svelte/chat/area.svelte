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

  // Bottom-most message in view, and how many pixels of it are hidden below the list
  interface ScrollPosition {
    seq: number;
    hiddenBelow: number;
  }

  // Scroll position of each room left this session, keyed by room ID
  const scrollPositions: Map<string, ScrollPosition> = new Map(
    JSON.parse(sessionStorage.getItem(KEY_SCROLL_POS) ?? "[]")
  );

  // Element of each shown message, keyed by seq
  const elements: Record<number, HTMLDivElement | null> = {};

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

  // Before the list changes rooms, stores the scroll position of the room being left
  $effect.pre(() => {
    const nextRoom = roomId;
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
      restore();
      return;
    }

    if (atBottom) {
      list.scrollTop = list.scrollHeight;
    }
  });

  // Keeps the list at the bottom as the messages grow, while it is at the bottom
  $effect(() => {
    if (content === undefined) {
      return;
    }

    const observer = new ResizeObserver(() => {
      if (list !== undefined && atBottom) {
        list.scrollTop = list.scrollHeight;
      }
    });

    observer.observe(content);
    return () => observer.disconnect();
  });


  // Methods

  // Saves the shown room's scroll position
  function savePositions(): void {

    // No room shown, or the list is not on the page yet
    if (shownRoom === null || list === undefined) {
      return;
    }

    const listBottom = list.getBoundingClientRect().bottom;
    let newest: ScrollPosition | null = null;

    // Messages are in seq order, oldest first
    for (const message of roomMessages) {
      const element = elements[message.seq];

      // Not on the page yet
      if (!element) {
        continue;
      }

      const rect = element.getBoundingClientRect();

      // Below the list's bottom edge, so the previous message was the bottom-most in view
      if (rect.top >= listBottom) {
        break;
      }

      newest = { seq: message.seq, hiddenBelow: rect.bottom - listBottom };
    }

    // Nothing in view was found, keep the previous save
    if (newest === null) {
      return;
    }

    scrollPositions.set(shownRoom, newest);
    sessionStorage.setItem(KEY_SCROLL_POS, JSON.stringify([...scrollPositions]));
  }

  // Restores the shown room's saved scroll position
  function restore(): void {

    // List is not on the page yet
    if (list === undefined) {
      return;
    }

    // Start at the bottom, kept when nothing is saved or every later message was deleted
    const saved = scrollPositions.get(roomId ?? "");
    list.scrollTop = list.scrollHeight;

    if (saved !== undefined && saved.seq < roomMessages[0].seq) {
      // Saved message is behind Load More, so show the oldest loaded one
      list.scrollTop = 0;
    } else if (saved !== undefined) {
      const listBottom = list.getBoundingClientRect().bottom;

      // Saved message, or the next one if it was deleted
      const match = roomMessages.find((message) => message.seq >= saved.seq);
      const element = match ? elements[match.seq] : null;

      // Put the message back with the same amount hidden below the list
      if (element) {
        const hiddenBelow = element.getBoundingClientRect().bottom - listBottom;
        list.scrollTop += hiddenBelow - saved.hiddenBelow;
      }
    }

    // Follow new messages only if the restored position is at the bottom
    onScroll();
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
    const unknown: (string | null)[] = [];
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
          bind:element={elements[message.seq]}
          author={users.getDisplayName(message.author_id)}
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
