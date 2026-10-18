<script lang="ts">
    import type { MessageResponse } from "@/bindings/routes/messages";
    import { parse } from "@/lib/markdown";
    import { LocalEvent } from "@/lib/events";
    import * as events from "@/lib/events";
    import Attachment from "./attachment.svelte";

    interface Props {
        message: MessageResponse;
        author: string;
        canEdit?: boolean;
        canDelete?: boolean;
        onEdit?: (body: string) => void;
        onDelete?: (skipConfirm: boolean) => void;
    }

    let { message, author, canEdit = false, canDelete = false, onEdit, onDelete }: Props = $props();

    let editing = $state(false);
    let draft = $state("");

    function startEdit(): void {
        draft = message.body ?? "";
        editing = true;
    }

    function saveEdit(): void {
        onEdit?.(draft);
        editing = false;
    }

    const createdAt = $derived(new Date(Number(message.created_at)));
    const editedAt = $derived(message.edited_at ? new Date(Number(message.edited_at)) : null);
    const deletedAt = $derived(message.deleted_at ? new Date(Number(message.deleted_at)) : null);
    const parts = $derived(message.body !== null ? parse(message.body) : []);

    let debugging = false;

    function formatTime(date: Date): string {
        const year = date.getFullYear();
        const month = String(date.getMonth() + 1).padStart(2, "0");
        const day = String(date.getDate()).padStart(2, "0");
        const hours = date.getHours() % 12 || 12;
        const minutes = String(date.getMinutes()).padStart(2, "0");
        const period = date.getHours() < 12 ? "AM" : "PM";

        return `${year}-${month}-${day} ${hours}:${minutes}${period}`;
    }
</script>

<style>
    .message {
        margin: 0 var(--control-padding) var(--control-padding);
        padding: var(--section-padding);
        border: 1px solid var(--border);
    }

    .message-header button {
        padding: 0;
        border: 0;
        background: none;
        font-weight: bold;
    }

    .message-body textarea {
        display: block;
        box-sizing: border-box;
        width: 100%;
        min-height: 4rem;
        resize: vertical;
    }

    .edit-actions {
        display: flex;
        justify-content: flex-end;
        gap: var(--inline-gap);
        margin-top: var(--row-gap);
    }

    .message-footer {
        display: flex;
        justify-content: space-between;
        align-items: center;
        margin-top: var(--row-gap);
    }

    .message-actions {
        display: flex;
        gap: var(--inline-gap);
    }

    .message-actions button {
        padding: 0;
        border: 0;
        background: none;
        color: var(--text-dim);
        font-size: 0.85rem;
        text-decoration: underline;
    }

    .message-footer time {
        color: var(--text-dim);
        font-size: 0.78rem;
        font-weight: bold;
    }

    .bold {
        font-weight: bold;
    }
    .italic {
        font-style: italic;
    }
    .strikethrough {
        text-decoration: line-through;
    }
</style>

<div class="message">
    
    <!-- Header -->
    <div class="message-header">
            <button onclick={() => events.dispatch(LocalEvent.user_selected, message.author_id)}>{author}</button>
            {#if editedAt}
            <span title="Edited at: {editedAt.toLocaleString()}"> (edited)</span>
            {/if}
            {#if debugging}
            <span>Sequence: {message.seq}</span>
            <span>ID: {message.id}</span>
            <span>Room ID: {message.room_id}</span>
            {/if}
        </div>

    <!-- Body -->
    <div class="message-body">

        {#if editing}
            <textarea bind:value={draft}></textarea>
            <div class="edit-actions">
                <button onclick={saveEdit}>Save</button>
                <button onclick={() => editing = false}>Cancel</button>
            </div>
        {:else}
        <!-- Parse Message Kind -->
        {#each parts as part}
            {#if part.kind === "text"}
                <span
                    class:bold={part.bold}
                    class:italic={part.italic}
                    class:strikethrough={part.strikethrough}
                >{part.content}</span>
            {:else if part.kind === "code"}
                <code>{part.content}</code>
            {:else if part.kind === "block"}
                <pre><code>{part.content}</code></pre>
            {:else if part.kind === "link"}
                <a href={part.href} target="_blank" rel="noopener noreferrer">{part.href}</a>
                <!-- Has recognized embed links, create player: TODO-->
            {:else if part.kind === "mention"}
                <span class="mention">@{part.userId}</span>
            {/if}
        {/each}
        {/if}

        <!-- Attachments -->
        {#each message.attachments as attachment}
            <Attachment {attachment}/>
        {/each}

    </div>

    <!-- Footer -->
    <div class="message-footer">
        
        <div class="message-actions">
            {#if canEdit}
            <button onclick={startEdit}>Edit</button>
            {/if}
            {#if canDelete}
            <button onclick={(event) => onDelete?.(event.ctrlKey)}>Delete</button>
            {/if}
        </div>

        <time datetime={createdAt.toISOString()}>{formatTime(createdAt)}</time>

        {#if debugging}
        <time datetime={deletedAt?.toISOString()}>{deletedAt?.toLocaleString()}</time>
        {/if}

    </div>

</div>