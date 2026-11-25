<script lang="ts">
  import RoomList from "./room_list.svelte";
  import Members from "./members.svelte";
  import * as call_participants from "@/lib/rtc/participants.svelte";

  interface Props {
    roomId: string | null;
    open: boolean;
  }

  let { roomId, open }: Props = $props();
</script>

<style>

  .column + .column {
    border-top: 1px solid var(--border);
  }

</style>

<div class="column grow">
  <RoomList hasCall={call_participants.hasCall}/>
</div>

<div class="column grow">
  <Members {roomId} {open}/>
</div>
