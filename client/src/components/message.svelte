<script lang="ts">
    import type { MessageResponse } from "@/bindings/MessageResponse";
    import { parse } from "@/lib/markdown";

    interface Props {
        message: MessageResponse;
        author: string;
        avatar?: string;
    } 

    let { message, author, avatar = "" }: Props = $props();

    const createdAt = $derived(new Date(Number(message.created_at)));
    const editedAt = $derived(message.edited_at ? new Date(Number(message.edited_at)) : null);
    const deletedAt = $derived(message.deleted_at ? new Date(Number(message.deleted_at)) : null);
    const parts = $derived(message.body !== null ? parse(message.body) : []);

    const avatar_width = "2em";
    const avatar_height = "2em";
    const avatar_autoplay = true;
    let debugging = false;
</script>

<style>
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
    
    <!-- Avatar -->
    <div class="avatar">
        {#if avatar.endsWith("mp4")}
            <video
                width={avatar_width}
                height={avatar_height}
                autoplay={avatar_autoplay}
            >
                <source src={avatar} type="video/mp4">
                Your browser does not support the video tag.
            </video>
        {:else}
            <img src={avatar} alt="User Avatar" width={avatar_width} height={avatar_height}>
        {/if}
    </div>

    <!-- Header -->
    <div class="message-header">
            <button>{author}</button>
            {#if editedAt}
            <span title="Edited at: {editedAt.toLocaleString()}">edited</span>
            {/if}
            {#if debugging}
            <span>Sequence: {message.seq}</span>
            <span>ID: {message.id}</span>
            <span>Room ID: {message.room_id}</span>
            {/if}
        </div>

    <!-- Body -->
    <div class="message-body">

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

        <!-- Attachments -->

    </div>

    <!-- Footer -->
    <div class="message-footer">
        
        <div class="message-actions">
            <button>Edit</button>
            <button>Delete</button>
        </div>

        <time datetime={createdAt.toISOString()}>{createdAt.toLocaleString()}</time>

        {#if debugging}
        <time datetime={deletedAt?.toISOString()}>{deletedAt?.toLocaleString()}</time>
        {/if}

    </div>

</div>