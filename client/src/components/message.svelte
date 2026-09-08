<script lang="ts">
    import type { MessageResponse } from '../bindings/MessageResponse';

    interface Props {
        message: MessageResponse;
        author: string;
        avatar?: string;
    }

    let { message, author, avatar = '' }: Props = $props();

    const createdAt = $derived(new Date(Number(message.created_at)));
    const editedAt = $derived(message.edited_at ? new Date(Number(message.edited_at)) : null);
    const deletedAt = $derived(message.deleted_at ? new Date(Number(message.deleted_at)) : null);

    const avatar_width = "2em";
    const avatar_height = "2em";
    const avatar_autoplay = true;
    let debugging = false;
</script>

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
        <!-- Regular text -> p -->
        <!-- Contains `abc` or ```abc``` parse as MD -->
        <!-- Has http/https create links -->
        <!-- Has recognized embed links, create player -->

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