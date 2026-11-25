<script lang="ts">
  import SectionBar from "./section_bar.svelte";
  import * as rooms from "@/lib/rooms/data.svelte";
  import * as users from "@/lib/users/data.svelte";
  import * as session from "@/lib/rtc/session.svelte";
  import * as call_participants from "@/lib/rtc/participants.svelte";

  interface Props {
    roomId: string | null;
  }

  let { roomId }: Props = $props();

  // The call already joined shows below instead
  const shown: boolean = $derived(roomId !== null && roomId !== session.getRoomId());
  const participants: string[] = $derived(call_participants.get(roomId));
  const room_name: string = $derived(roomId === null ? "" : rooms.get(roomId)?.name ?? "");

  function join(): void {
    if (roomId !== null) {
      session.join(roomId);
    }
  }
</script>

<style>

  .preview {
    flex: none;
    padding-bottom: 0.25rem;
  }

  .name, .empty {
    display: block;
    padding: 0.25rem 0.5rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .empty {
    color: var(--text-dim);
  }

</style>

{#if shown}
  <div class="preview">
    <SectionBar label="Preview" detail={room_name}>
      <button class="is-action is-compact" onclick={join}>Join</button>
    </SectionBar>

    {#each participants as user_id (user_id)}
      <span class="name">{users.getDisplayName(user_id)}</span>
    {:else}
      <span class="empty">No one in call</span>
    {/each}
  </div>
{/if}
