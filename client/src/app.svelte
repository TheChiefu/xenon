<script lang="ts">
    import Message from "./components/message.svelte";
    import type { MessageResponse } from "./bindings/MessageResponse";

    const sampleMessage: MessageResponse = {
        seq: 1n,
        id: '00000000-0000-0000-0000-000000000000',
        room_id: '00000000-0000-0000-0000-000000000000',
        author_id: '00000000-0000-0000-0000-000000000000',
        body: 'Hello world this is a **bold** message with a [link](https://example.com) and a `code snippet`.',
        created_at: BigInt(Date.now()),
        edited_at: null,
        deleted_at: null,
        attachments: [],
    };
</script>


<Message message={sampleMessage} author="Test User"></Message>
