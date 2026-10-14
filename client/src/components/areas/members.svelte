<script lang="ts">
  import { Status } from "@/bindings/shared";
  import type { GameActivity } from "@/bindings/sockets/events";
  import { ICONS_DEVICE, ICONS_PLATFORM, LABELS_DEVICE, LABELS_GAME_PRESENCE, LABELS_PLATFORM } from "@/lib/labels";
  import { failureText } from "@/lib/utils";
  import { listMembers } from "@/lib/api/rooms";
  import { getSelectedRoom } from "@/lib/session.svelte";
  import { getActivityOfUser, getAvatarOfUser, getBannerOfUser, getDisplayNameOfUser, getPresenceOfUser, getUser, loadProfileFilesOfUser, loadUsers } from "@/lib/users.svelte";
  import { animatePhoto, playOnHover } from "@/lib/settings.svelte";
  import { Pane } from "@/lib/panes.svelte";
  import PaneArea from "@/components/pane.svelte";

  // Order in list of status in terms of position
  const ORDER_STATUS: Record<Status, number> = {
    [Status.online]: 0,
    [Status.busy]: 1,
    [Status.away]: 2,
    [Status.offline]: 3,
  };

  let member_ids: string[] = $state([]);
  const ordered_ids: string[] = $derived(member_ids.toSorted(compareMembers));
  let failure: string | null = $state(null);

  // Loads members of a selected room
  $effect(() => {
    const roomId = getSelectedRoom();
    
    // Clear member list if no room is selected
    if (roomId === null) {
      member_ids = [];
      return;
    }

    load(roomId);
  });

  $effect(() => {
    for (const id of member_ids) {
      loadProfileFilesOfUser(id);
    }
  });

  // Gets room members IDs
  async function load(roomId: string): Promise<void> {
    try {
      const entries = await listMembers(roomId);
      const ids: string[] = entries.map((entry) => entry.user_id);

      await loadUsers(ids);

      member_ids = [];
      for (const id of ids) {
        if (getUser(id) !== undefined) {
          member_ids.push(id);
        }
      }

      failure = null;
    } catch (error) {
      member_ids = [];
      failure = failureText(error);
    }
  }

  // Game title, followed by activity when reported
  function activityText(activity: GameActivity): string {
    const title = activity.title ?? LABELS_GAME_PRESENCE[activity.status];

    if (activity.detail === null) {
      return title;
    }

    return `${title} · ${activity.detail}`;
  }

  // Compare to members to see who is before or after the other
  function compareMembers(a: string, b: string): number {

    // Determine state difference
    const status_a = ORDER_STATUS[getPresenceOfUser(a).status];
    const status_b = ORDER_STATUS[getPresenceOfUser(b).status];
    if (status_a !== status_b) {
      return status_a - status_b;
    }

    // Sort by display name
    const name_a = getDisplayNameOfUser(a);
    const name_b = getDisplayNameOfUser(b);
    return name_a.localeCompare(name_b);
  }

</script>

<style>

  /* Pane */

  /* Header */

  .header {
    display: flex;
    flex-direction: row;
    flex: none;
    padding: 0.5rem;
    align-items: center;
  }

  .header h3 {
    margin-right: auto;
    font-size: 1rem;
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

  .member {
    --avatar-size: 3rem;
    position: relative;
    height: var(--avatar-size);
    display: flex;
    align-items: center;
    padding: 0;
    border: 0;
    background-color: var(--component);
    color: var(--text);
    font: inherit;
    text-align: left;
    overflow: hidden;

    /* Keeps the banner's negative z-index inside the row */
    isolation: isolate;
  }

  .member:hover {
    background-color: var(--component-hover);
  }

  /* Behind the row's contents, above the row's own background */
  .banner {
    position: absolute;
    inset: 0;
    z-index: -1;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .avatar {
    flex: none;
    width: var(--avatar-size);
    height: var(--avatar-size);
    object-fit: cover;
    filter:
      drop-shadow(1px 0 0 rgba(0, 0, 0, 1))
      drop-shadow(-1px 0 0 rgba(0, 0, 0, 1))
      drop-shadow(0 1px 0 rgba(0, 0, 0, 1))
      drop-shadow(0 -1px 0 rgba(0, 0, 0, 1));
  }

  .text {
    display: grid;
    flex: 1;
    min-width: 0;
    padding: 0 0.5rem;
  }

  .name, .game, .activity {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .game {
    /* Smaller than the name, so the platform glyph is too */
    --icon-size: 1rem;
    display: flex;
    align-items: center;
    gap: 0.25rem;
    color: var(--text-dim);
    font-size: 0.78rem;
  }

  
  .device {
    flex: none;
    margin-right: 0.5rem;
    color: rgba(255, 255, 255, 1);
    opacity: 0.8;
  }

  /* The artwork reaches the viewBox edges, so the stroke needs room past it */
  .device :global(svg) {
    overflow: visible;
  }

  .device :global(svg *) {
    stroke: rgba(0, 0, 0, 1);
    stroke-width: 1px;
    paint-order: stroke;
    vector-effect: non-scaling-stroke;
  }

  /* Status Bar */

  .status {
    width: var(--status-bar-width);
    flex: none;
    align-self: stretch;
  }


  .online  { background: var(--status-online); }
  .busy    { background: var(--status-busy); }
  .away    { background: var(--status-away); }
  .offline { background: var(--status-offline); }

</style>

<!-- Member Pane -->
<PaneArea pane={Pane.members}>

  <!-- Display Header -->
  <div class="header">
    <h3>Members: {member_ids.length}</h3>
  </div>

  {#if failure !== null}
    <p class="failure">Failure: {failure}</p>
  {/if}

  <!-- Member List -->
  <div class="list">
    {#each ordered_ids as id (id)}
      {@const avatar = getAvatarOfUser(id)}
      {@const banner = getBannerOfUser(id)}
      {@const presence = getPresenceOfUser(id)}
      {@const activity = getActivityOfUser(id)}
      <button class="member" {@attach playOnHover}>
        {#if banner !== undefined}
          {#if banner.mime.startsWith("video/")}
            <video class="banner" src={banner.url} {@attach animatePhoto}></video>
          {:else}
            <img class="banner" src={banner.url} alt=""/>
          {/if}
        {/if}

        {#if avatar === undefined}
          <span class="avatar"></span>
        {:else if avatar.mime.startsWith("video/")}
          <video class="avatar" src={avatar.url} {@attach animatePhoto}></video>
        {:else}
          <img class="avatar" src={avatar.url} alt=""/>
        {/if}

        <span class="text text-outline">
          <span class="name">{getDisplayNameOfUser(id)}</span>

          {#if activity !== undefined}
            <span class="game">
              <span class="icon" aria-hidden="true" title={LABELS_PLATFORM[activity.platform]}>
                {@html ICONS_PLATFORM[activity.platform]}
              </span>
              <span class="activity" title={activityText(activity)}>{activityText(activity)}</span>
            </span>
          {/if}
        </span>

        {#if presence.device !== null}
          <span class="icon device" aria-hidden="true" title={LABELS_DEVICE[presence.device]}>
            {@html ICONS_DEVICE[presence.device]}
          </span>
        {/if}

        <span class="status {presence.status}"></span>
      </button>
    {/each}
  </div>
</PaneArea>
