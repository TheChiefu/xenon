<script lang="ts">
  import { Device, type GameActivity } from "@/bindings/sockets/events";
  import { Status } from "@/bindings/shared";
  import { ICONS_DEVICE, ICONS_PLATFORM, LABELS_DEVICE, LABELS_PLATFORM } from "@/lib/labels";
  import { fetchFileBlob, type FetchedFile } from "@/lib/utils";
  import { listMembers } from "@/lib/api/rooms";
  import { getSelectedRoom } from "@/lib/rooms/data.svelte";
  import { getUser, loadUsers } from "@/lib/users.svelte";
  import { animatePhoto, playOnHover } from "@/lib/settings.svelte";

  // What one row shows of a member
  interface Member {
    id: string;
    display_name: string;
    avatar_file_id: string | null;
    banner_file_id: string | null;
    avatar: FetchedFile | null;
    banner: FetchedFile | null;
    status: Status;
    device: Device | null;
    game: GameActivity | null;
  }

  let members: Member[] = $state([]);
  let failure: string | null = $state(null);

  $effect(() => {
    const roomId = getSelectedRoom();
    if (roomId === null) {
      members = [];
      return;
    }

    load(roomId);
  });

  async function load(roomId: string): Promise<void> {
    try {
      const entries = await listMembers(roomId);
      const ids: string[] = entries.map((entry) => entry.user_id);

      await loadUsers(ids);

      members = ids
        .map((id) => getUser(id))
        .filter((user) => user !== undefined)
        .map((user) => ({
          id: user.id,
          display_name: user.display_name,
          avatar_file_id: user.avatar_file_id,
          banner_file_id: user.banner_file_id,
          avatar: null,
          banner: null,
          status: Status.offline,

          // TEMPORARY: stands in until the socket reports what a member is on
          device: Device.macos,
          game: null,
        }));

      failure = null;

      // Fetch the avatar and banner of every member
      for (const member of members) {
        if (member.avatar_file_id !== null) {
          member.avatar = await fetchFileBlob(member.avatar_file_id);
        }

        if (member.banner_file_id !== null) {
          member.banner = await fetchFileBlob(member.banner_file_id);
        }
      }
    } catch (error) {
      members = [];
      failure = error instanceof Error ? error.message : String(error);
    }
  }

  // Where each status sits in the list
  const ORDER_STATUS: Record<Status, number> = {
    [Status.online]: 0,
    [Status.busy]: 1,
    [Status.away]: 2,
    [Status.offline]: 3,
  };

  // Status first, then alphabetical within it, redone as members come and go
  const ordered: Member[] = $derived([...members].sort((a, b) =>
    ORDER_STATUS[a.status] - ORDER_STATUS[b.status]
    || a.display_name.localeCompare(b.display_name)
  ));
</script>

<style>

  /* Pane */

  .members {
    width: 18rem;
    display: flex;
    flex-direction: column;
    flex: none;
    border-right: 1px solid var(--border);
    background: rgba(30,30,30, 0.5);
    overflow: hidden;
  }

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

  .name, .game {
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
<div class="members">

  <!-- Display Header -->
  <div class="header">
    <h3>Members: {members.length}</h3>
  </div>

  {#if failure !== null}
    <p class="failure">Failure: {failure}</p>
  {/if}

  <!-- Member List -->
  <div class="list">
    {#each ordered as member (member.id)}
      {@const avatar = member.avatar}
      {@const banner = member.banner}
      <button class="member" {@attach playOnHover}>
        {#if banner !== null}
          {#if banner.mime.startsWith("video/")}
            <video class="banner" src={banner.url} {@attach animatePhoto}></video>
          {:else}
            <img class="banner" src={banner.url} alt=""/>
          {/if}
        {/if}

        {#if avatar === null}
          <span class="avatar"></span>
        {:else if avatar.mime.startsWith("video/")}
          <video class="avatar" src={avatar.url} {@attach animatePhoto}></video>
        {:else}
          <img class="avatar" src={avatar.url} alt=""/>
        {/if}

        <span class="text text-outline">
          <span class="name">{member.display_name}</span>

          {#if member.game !== null}
            <span class="game">
              <span class="icon" aria-hidden="true" title={LABELS_PLATFORM[member.game.platform]}>
                {@html ICONS_PLATFORM[member.game.platform]}
              </span>
              {member.game.title ?? member.game.activity}
            </span>
          {/if}
        </span>

        {#if member.device !== null}
          <span class="icon device" aria-hidden="true" title={LABELS_DEVICE[member.device]}>
            {@html ICONS_DEVICE[member.device]}
          </span>
        {/if}

        <span class="status {member.status}"></span>
      </button>
    {/each}
  </div>
</div>
