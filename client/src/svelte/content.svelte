<script lang="ts">
  import RoomBar from "./bars/room.svelte";
  import StreamBar from "./bars/stream.svelte";
  import Chat from "./chat/area.svelte";
  import Stream from "./stream/area.svelte";
  import { LocalEvent } from "@/lib/events";
  import * as events from "@/lib/events";

  interface Props {
    roomId: string | null;
  }

  let { roomId }: Props = $props();

  let showing_stream: boolean = $state(false);

  function showChat(): void {
    showing_stream = false;
  }

  function showStream(): void {
    showing_stream = true;
  }

  // Switch tabs when a screen share is watched or closed
  events.subscribe(LocalEvent.stream_requested, showStream);
  events.subscribe(LocalEvent.chat_requested, showChat);
</script>

<div class="chat column">
  <div class="row">
    <button class="is-tab" class:is-active={!showing_stream} onclick={showChat}>Chat</button>
    <button class="is-tab" class:is-active={showing_stream} onclick={showStream}>Stream</button>
  </div>

  {#if showing_stream}
    <StreamBar/>
    <Stream/>
  {:else}
    {#if roomId !== null}
      <RoomBar {roomId}/>
    {/if}
    <Chat {roomId}/>
  {/if}
</div>
