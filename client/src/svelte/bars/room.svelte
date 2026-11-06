<script lang="ts">
  import { LocalEvent } from "@/lib/events";
  import * as events from "@/lib/events";
  import * as rooms from "@/lib/rooms/data.svelte";
  import * as selection from "@/lib/rooms/selection.svelte";
  import icon_back from "@/assets/icons/back.svg?raw";
  import icon_copy from "@/assets/icons/copy.svg?raw";
  import icon_gear from "@/assets/icons/gear.svg?raw";

  interface Props {
    roomId: string;
  }

  let { roomId }: Props = $props();

  function copyId(): void {
    navigator.clipboard.writeText(roomId);
  }

  function openSettings(): void {
    events.dispatch(LocalEvent.room_settings_requested, roomId);
  }
</script>

<div class="row header-bar">
  <h3 class="grow">{rooms.getName(roomId)}</h3>

  <button class="icon-btn" aria-label="Close room" title="Close room" onclick={() => selection.select(null)}>
    <span class="icon" aria-hidden="true">{@html icon_back}</span>
  </button>
  <button class="icon-btn" aria-label="Copy room ID" title="Copy room ID" onclick={copyId}>
    <span class="icon" aria-hidden="true">{@html icon_copy}</span>
  </button>
  <button class="icon-btn" aria-label="Room settings" title="Room settings" onclick={openSettings}>
    <span class="icon" aria-hidden="true">{@html icon_gear}</span>
  </button>
</div>
