<script lang="ts">
    import Message from "./components/message.svelte";
    import ServerInfo from "./components/areas/server_info.svelte";
    import type { MessageResponse } from "./bindings/routes/messages";

    const sampleMessage: MessageResponse = {
        seq: 1n,
        id: '00000000-0000-0000-0000-000000000000',
        room_id: '00000000-0000-0000-0000-000000000000',
        author_id: '00000000-0000-0000-0000-000000000000',
        body: 'Hello world, this is a **bold** and *italic* message with a link to https://example.com and a `code snippet`.',
        created_at: BigInt(Date.now()),
        edited_at: null,
        deleted_at: null,
        attachments: [],
    };
</script>


<ServerInfo />
<Message message={sampleMessage} author="Test User"></Message>
