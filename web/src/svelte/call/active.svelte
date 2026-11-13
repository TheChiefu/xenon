<script lang="ts">
  import SectionBar from "./section_bar.svelte";
  import { Permission } from "@/bindings/types/rooms";
  import * as members from "@/lib/rooms/members.svelte";
  import * as rooms from "@/lib/rooms/data.svelte";
  import * as users from "@/lib/users/data.svelte";
  import * as profile from "@/lib/users/profile.svelte";
  import * as session from "@/lib/rtc/session.svelte";
  import * as microphones from "@/lib/rtc/microphones.svelte";
  import * as call_participants from "@/lib/rtc/participants.svelte";
  import * as levels from "@/lib/rtc/levels.svelte";
  import * as screens from "@/lib/rtc/screens.svelte";
  import { LocalEvent } from "@/lib/events";
  import * as events from "@/lib/events";

  import icon_headphones from "@assets/icons/headphones.svg?raw";
  import icon_headphones_muted from "@assets/icons/headphones-muted.svg?raw";
  import icon_mic from "@assets/icons/mic.svg?raw";
  import icon_mic_muted from "@assets/icons/mic-muted.svg?raw";
  import icon_monitor from "@assets/icons/monitor.svg?raw";
  import icon_volume from "@assets/icons/volume.svg?raw";
  import icon_volume_off from "@assets/icons/volume-off.svg?raw";

  const self_id: string = $derived(profile.get()?.id ?? "");
  const room_id: string | null = $derived(session.getRoomId());

  // Load members of the call's room for names and permissions
  $effect(() => {
    if (room_id !== null) {
      members.load(room_id);
    }
  });

  function toggleMic(): void {
    session.setMicEnabled(!session.getMicEnabled());
  }

  function toggleSound(): void {
    microphones.setSoundEnabled(!microphones.getSoundEnabled());
  }

  function toggleUserMuted(user_id: string): void {
    microphones.setUserMuted(user_id, !microphones.isUserMuted(user_id));
  }

  // Apply the slider's position as the participant's volume
  function changeVolume(user_id: string, event: Event): void {
    const slider = event.currentTarget as HTMLInputElement;
    microphones.setVolume(user_id, Number(slider.value));
  }

  function toggleScreen(): void {
    session.setScreenEnabled(!session.getScreenEnabled()).catch(console.warn);
  }
</script>

<style>

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
    flex: none;
    height: 2rem;
    display: flex;
    align-items: center;
    gap: 0.25rem;
    padding-right: 0.5rem;
    margin-bottom: 1px;
    background-color: var(--component);
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
    min-width: 4rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .volume {
    width: 8rem;
    min-width: 3rem;
    flex: 0 1 auto;
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

<SectionBar label="Active" detail={rooms.getName(room_id)}/>

{#if session.getFailure() !== null}
  <p class="failure">{session.getFailure()}</p>
{/if}

<!-- Participants -->
{#if room_id === null}
  <p class="empty">Not in a call</p>
{:else}
  <div class="list">
    {#each call_participants.getSorted(room_id) as user_id (user_id)}
      {@const name = users.getDisplayName(user_id)}
      <div class="participant" class:speaking={levels.isSpeaking(user_id)}>
        <span class="strip"></span>
        <span class="name" title={name}>{name}</span>

        {#if user_id !== self_id}
          {#if screens.isSharing(user_id)}
            <button
              class="call-btn"
              class:is-on={screens.getWatching() === user_id}
              aria-label="Watch {name}"
              title="Watch"
              onclick={() => {
                screens.watch(user_id);
                events.dispatch(LocalEvent.stream_requested);
              }}
            >
              <span class="icon" aria-hidden="true">{@html icon_monitor}</span>
            </button>
          {/if}
          <input
            class="volume"
            type="range"
            min="0"
            max={microphones.VOLUME_MAX}
            value={microphones.getVolume(user_id)}
            oninput={(event) => changeVolume(user_id, event)}
            aria-label="Volume of {name}"
            title="Volume"
          />
          <button
            class="call-btn"
            class:is-muted={microphones.isUserMuted(user_id)}
            aria-label="Mute {name} for you"
            title="Mute for you"
            onclick={() => toggleUserMuted(user_id)}
          >
            <span class="icon" aria-hidden="true">{@html microphones.isUserMuted(user_id) ? icon_volume_off : icon_volume}</span>
          </button>

          {#if members.hasPermission(room_id, self_id, Permission.mute)}
            <button
              class="call-btn"
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
    class:is-muted={!microphones.getSoundEnabled()}
    aria-label="Toggle sound"
    title="Toggle Sound"
    onclick={toggleSound}
  >
    <span class="icon" aria-hidden="true">{@html microphones.getSoundEnabled() ? icon_headphones : icon_headphones_muted}</span>
  </button>

  {#if room_id !== null}
    <button
      class="call-btn"
      class:is-on={session.getScreenEnabled()}
      aria-label="Toggle screenshare"
      title="Toggle Screenshare"
      onclick={toggleScreen}
    >
      <span class="icon" aria-hidden="true">{@html icon_monitor}</span>
    </button>
    <button class="end is-danger" onclick={session.leave}>Leave</button>
  {/if}
</div>
