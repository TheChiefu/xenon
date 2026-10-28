<script lang="ts">
  import { Permission } from "@/bindings/types/rooms";
  import * as members from "@/lib/rooms/members.svelte";
  import * as rooms from "@/lib/rooms/data.svelte";
  import * as users from "@/lib/users/data.svelte";
  import * as profile from "@/lib/users/profile.svelte";
  import * as session from "@/lib/call/session.svelte";
  import * as call_participants from "@/lib/call/participants.svelte";
  import * as levels from "@/lib/call/levels.svelte";

  import icon_headphones from "@/assets/icons/headphones.svg?raw";
  import icon_headphones_muted from "@/assets/icons/headphones-muted.svg?raw";
  import icon_mic from "@/assets/icons/mic.svg?raw";
  import icon_mic_muted from "@/assets/icons/mic-muted.svg?raw";
  import icon_camera from "@/assets/icons/camera.svg?raw";
  import icon_monitor from "@/assets/icons/monitor.svg?raw";
  import icon_volume from "@/assets/icons/volume.svg?raw";
  import icon_volume_off from "@/assets/icons/volume-off.svg?raw";

  interface Props {
    roomId: string | null;
  }

  let { roomId }: Props = $props();

  interface ParticipantState {
    mic_on: boolean;
    video_on: boolean;
    moderator_muted: boolean;
    watching: boolean;
  }

  interface Participant {
    user_id: string;
    state: ParticipantState;
  }

  // Shown for every participant until media state is sent
  const default_state: ParticipantState = {
    mic_on: true,
    video_on: false,
    moderator_muted: false,
    watching: false,
  };

  const self_id: string = $derived(profile.get()?.id ?? "");
  const joined: boolean = $derived(session.getRoomId() !== null);

  // The call joined, otherwise the selected room
  const shown_room_id: string | null = $derived(session.getRoomId() ?? roomId);

  const participants: Participant[] = $derived.by(() => {
    if (shown_room_id === null) {
      return [];
    }

    const result: Participant[] = [];

    for (const user_id of call_participants.get(shown_room_id).toSorted(compareNames)) {
      result.push({ user_id, state: default_state });
    }

    return result;
  });

  const room_name: string = $derived(shown_room_id === null ? "" : rooms.get(shown_room_id)?.name ?? "");
  const can_mute: boolean = $derived(shown_room_id !== null && members.getPermissions(shown_room_id, self_id).includes(Permission.mute));

  let camera_on: boolean = $state(false);
  let screen_on: boolean = $state(false);

  // Load members of shown room for names and permissions
  $effect(() => {
    if (shown_room_id === null) {
      return;
    }

    members.load(shown_room_id);
  });

  // Compare two participants to see who is before or after the other
  function compareNames(a: string, b: string): number {
    const name_a = users.getDisplayName(a);
    const name_b = users.getDisplayName(b);
    return name_a.localeCompare(name_b);
  }

  function toggleMic(): void {
    session.setMicEnabled(!session.getMicEnabled());
  }

  function toggleSound(): void {
    session.setSoundEnabled(!session.getSoundEnabled());
  }

  function toggleUserMuted(user_id: string): void {
    session.setUserMuted(user_id, !session.isUserMuted(user_id));
  }

  // Apply the slider's position as the participant's volume
  function changeVolume(user_id: string, event: Event): void {
    const slider = event.currentTarget as HTMLInputElement;
    session.setVolume(user_id, Number(slider.value));
  }

  // Video carries camera or screen, one at a time
  function toggleCamera(): void {
    camera_on = !camera_on;
    screen_on = false;
  }

  function toggleScreen(): void {
    screen_on = !screen_on;
    camera_on = false;
  }

  function join(): void {
    if (roomId !== null) {
      session.join(roomId);
    }
  }
</script>

<style>

  /* Header */

  .header {
    flex: none;
    padding: 0.5rem;
    font-size: 1rem;
  }

  .empty {
    flex: 1;
    display: grid;
    place-items: center;
    color: var(--text-dim);
  }

  .failure {
    flex: none;
    margin: 0;
    padding: 0.5rem;
    color: var(--danger);
    font-size: 0.85rem;
  }

  /* List */

  .list {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    overflow-y: auto;
  }

  /* Rows */

  .participant {
    height: 2rem;
    display: flex;
    align-items: center;
    gap: 0.25rem;
    padding-right: 0.5rem;
    margin-bottom: 1px;
    background-color: var(--component);
  }

  .preview {
    padding: 0.25rem 0.5rem;
  }

  .strip {
    width: var(--status-bar-width);
    flex: none;
    align-self: stretch;
  }

  .participant.speaking .strip {
    background: var(--status-online);
  }

  .name {
    flex: 1;
    min-width: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .indicator {
    --icon-size: 1rem;
    flex: none;
    color: var(--text-dim);
  }

  .indicator.moderator {
    color: var(--danger);
  }

  .volume {
    width: 4.5rem;
    flex: none;
    padding: 0;
    border: 0;
    cursor: pointer;
  }

  /* Controls */

  .controls {
    flex: none;
    display: flex;
    align-items: center;
    gap: 0.25rem;
    padding: 0.5rem;
    border-top: 1px solid var(--border);
  }

  .controls .end {
    margin-left: auto;
  }

  /* Buttons */

  .call-btn {
    width: 2.125rem;
    height: 2.125rem;
    padding: 0;
    flex: none;
    display: grid;
    place-items: center;
    border: 0;
    background: transparent;
    color: inherit;
  }

  .call-btn:hover {
    background: var(--component-hover);
  }

  .call-btn.is-muted,
  .call-btn.is-muted:hover {
    background: var(--btn-muted);
  }

  .call-btn.is-on,
  .call-btn.is-on:hover {
    background: var(--btn-active);
  }

</style>

<!-- Call Header -->
{#if participants.length > 0}
  <h3 class="header">{room_name} · {participants.length} in call</h3>
{/if}

{#if session.getFailure() !== null}
  <p class="failure">{session.getFailure()}</p>
{/if}

<!-- Participants -->
{#if participants.length === 0}
  <p class="empty">No calls active</p>
{:else if !joined}
  <div class="list">
    {#each participants as participant (participant.user_id)}
      <span class="preview">{users.getDisplayName(participant.user_id)}</span>
    {/each}
  </div>
{:else}
  <div class="list">
    {#each participants as participant (participant.user_id)}
      {@const name = users.getDisplayName(participant.user_id)}
      {@const is_self = participant.user_id === self_id}
      <div class="participant" class:speaking={levels.isSpeaking(participant.user_id)}>
        <span class="strip"></span>
        <span class="name" title={name}>{name}</span>

        {#if !participant.state.mic_on}
          <span class="icon indicator" aria-label="Microphone off" title="Microphone off">{@html icon_mic_muted}</span>
        {/if}
        {#if participant.state.moderator_muted}
          <span class="icon indicator moderator" aria-label="Muted by a moderator" title="Muted by a moderator">{@html icon_mic_muted}</span>
        {/if}

        {#if !is_self}
          {#if participant.state.video_on}
            <button
              class="call-btn"
              class:is-on={participant.state.watching}
              aria-label="Watch {name}"
              title="Watch"
            >
              <span class="icon" aria-hidden="true">{@html icon_camera}</span>
            </button>
          {/if}
          <input
            class="volume"
            type="range"
            min="0"
            max={session.VOLUME_MAX}
            value={session.getVolume(participant.user_id)}
            oninput={(event) => changeVolume(participant.user_id, event)}
            aria-label="Volume of {name}"
            title="Volume"
          />
          <button
            class="call-btn"
            class:is-muted={session.isUserMuted(participant.user_id)}
            aria-label="Mute {name} for you"
            title="Mute for you"
            onclick={() => toggleUserMuted(participant.user_id)}
          >
            <span class="icon" aria-hidden="true">{@html session.isUserMuted(participant.user_id) ? icon_volume_off : icon_volume}</span>
          </button>

          {#if can_mute}
            <button
              class="call-btn"
              class:is-muted={participant.state.moderator_muted}
              aria-label="Mute {name} for everyone"
              title="Mute for everyone"
            >
              <span class="icon" aria-hidden="true">{@html icon_mic_muted}</span>
            </button>
          {/if}
        {/if}
      </div>
    {/each}
  </div>
{/if}

<!-- Own Controls -->
<div class="controls">
  <button
    class="call-btn"
    class:is-muted={!session.getMicEnabled()}
    aria-label="Toggle microphone"
    title="Toggle Microphone"
    onclick={toggleMic}
  >
    <span class="icon" aria-hidden="true">{@html session.getMicEnabled() ? icon_mic : icon_mic_muted}</span>
  </button>
  <button
    class="call-btn"
    class:is-muted={!session.getSoundEnabled()}
    aria-label="Toggle sound"
    title="Toggle Sound"
    onclick={toggleSound}
  >
    <span class="icon" aria-hidden="true">{@html session.getSoundEnabled() ? icon_headphones : icon_headphones_muted}</span>
  </button>

  {#if joined}
    <button
      class="call-btn"
      class:is-on={camera_on}
      aria-label="Toggle camera"
      title="Toggle Camera"
      onclick={toggleCamera}
    >
      <span class="icon" aria-hidden="true">{@html icon_camera}</span>
    </button>
    <button
      class="call-btn"
      class:is-on={screen_on}
      aria-label="Toggle screenshare"
      title="Toggle Screenshare"
      onclick={toggleScreen}
    >
      <span class="icon" aria-hidden="true">{@html icon_monitor}</span>
    </button>
    <button class="end is-danger" onclick={session.leave}>Leave</button>
  {:else}
    <button class="end is-action" disabled={roomId === null} onclick={join}>Join</button>
  {/if}
</div>
