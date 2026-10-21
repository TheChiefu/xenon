<script lang="ts">
  import type { LinkedAccountResponse } from "@/bindings/routes/users";
  import { Platform, type GlobalRole, type Status } from "@/bindings/shared";
  import type { Device, GameActivity } from "@/bindings/sockets/events";
  import { ICONS_PLATFORM, LABELS_DEVICE, LABELS_PLATFORM, LABELS_ROLE, LABELS_STATUS } from "@/lib/labels";
  import type { FetchedFile } from "@/lib/utils";
  import * as settings from "@/lib/client/settings.svelte";
  import * as menu from "@/lib/display/menu.svelte";
  

  interface Props {
    id: string;
    display_name: string;
    username: string;
    description: string;
    status?: Status | null;
    device?: Device | null;
    game?: GameActivity | null;
    role: GlobalRole;
    links: LinkedAccountResponse[];
    created_at: string;
    avatar?: FetchedFile | null;
    banner?: FetchedFile | null;
  }
  let {
    id,
    display_name,
    username,
    description,
    status = null,
    device = null,
    game = null,
    role,
    links,
    created_at,
    avatar = null,
    banner = null,
  }: Props = $props();

  // Provide context menu on xbox handle click
  function openXbox(event: MouseEvent, handle: string) {
    const TRUE_ACHIEVEMENTS = "https://www.trueachievements.com/gamer/"
    const XBOX_PLAY = "https://www.xbox.com/en-US/play/user/"

    // Different user related sites
    menu.open(event, [
      {
        label: "TrueAchievements",
        action: () => window.open(TRUE_ACHIEVEMENTS + handle.replaceAll(" ", "+"), "_blank", "noopener")
      },
      {
        label: "Xbox",
        action: () => window.open(XBOX_PLAY + encodeURIComponent(handle), "_blank", "noopener")
      },
    ]);
  }

  function openSteam() {
    window.open("", "_blank", "noopener");
  }

  const OPEN_PLATFORM: Record<Platform, (event: MouseEvent, handle: string) => void> = {
    [Platform.xbox]: openXbox,
    [Platform.steam]: openSteam,
  };
</script>

<style>
  .card {
    width: 18rem;
    border: 1px solid var(--border);
    background: var(--component);
    color: var(--text);
    overflow: hidden;
  }

  .header {
    position: relative;
    display: flex;
    align-items: center;
    gap: 0.75rem;
    padding: 1rem 0.75rem;
    background: var(--border);
  }

  .identity {
    flex: 1;
    position: relative;
    text-align: right;
    text-shadow:
      1px 0 0 var(--text-shadow),
      -1px 0 0 var(--text-shadow),
      0 1px 0 var(--text-shadow),
      0 -1px 0 var(--text-shadow);
  }

  .avatar {
    position: relative;
    flex-shrink: 0;
    width: 5rem;
    height: 5rem;
  }

  .media {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .body {
    padding: 0 1rem 1rem;
  }

  .body > :first-child {
    margin-top: 0;
  }

  .name {
    font-size: 1.25rem;
    font-weight: bold;
  }

  .username {
    color: var(--text-dim);
    font-size: 0.85rem;
    cursor: pointer;
    padding: 0.4rem 0;
    text-align: right;
  }

  button {
    background: none;
  }

  button:hover {
    color: var(--link);
    background: none;
  }

  button:active {
    color: var(--link-active);
    background: none;
  }

  .description, .meta {
    margin-top: 0.75rem;
    padding-top: 0.75rem;
    border-top: 1px solid var(--border);
  }

  .description {
    white-space: pre-wrap;
    overflow-wrap: anywhere;
  }

  .meta {
    display: grid;
    grid-template-columns: 2fr 3fr;
    align-items: center;
    row-gap: 0.35rem;
    font-size: 0.85rem;
  }

  dt {
    color: var(--text-dim);
  }

  dd {
    margin-left: 0.25rem;
  }

  p {
    margin-top: 0.15rem;
    margin-bottom: 0.15rem;
  }

  .link {
    display: flex;
    align-items: center;
    padding: 0;
    gap: 0.4rem;
  }

</style>

<div class="card">
  <div class="header" {@attach settings.playOnHover}>
    {#if banner !== null}
      {#if banner.mime.startsWith("video/")}
        <video class="media" src={banner.url} {@attach settings.animatePhoto}></video>
      {:else}
        <img class="media" src={banner.url} alt=""/>
      {/if}
    {/if}

    <div class="avatar">
      {#if avatar !== null}
        {#if avatar.mime.startsWith("video/")}
          <video class="media" src={avatar.url} {@attach settings.animatePhoto}></video>
        {:else}
          <img class="media" src={avatar.url} alt=""/>
        {/if}
      {/if}
    </div>
    <div class="identity">
      <p class="name">{display_name}</p>
      <button
        class="username inherit"
        onclick={() => navigator.clipboard.writeText(id)}
        title="Copy User ID"
      >@{username}</button>
    </div>
  </div>

  <div class="body">

    {#if description}
      <p class="description">{description}</p>
    {/if}

    <dl class="meta">
      {#if status !== null}
        <dt>Status</dt>
        <dd>{LABELS_STATUS[status]}{device === null ? "" : ` · ${LABELS_DEVICE[device]}`}</dd>
      {/if}

      {#if game !== null}
        <dt>Game Activity</dt>
        <dd>
          <span class="link">
            <span class="icon" title={LABELS_PLATFORM[game.platform]}>{@html ICONS_PLATFORM[game.platform]}</span>
            {game.title ?? game.detail ?? game.status}
          </span>
        </dd>
      {/if}

      <dt>Global Role</dt>
      <dd>{LABELS_ROLE[role]}</dd>
      <dt>Linked Accounts</dt>
      <dd>
        {#each links as link}
          <button type="button" class="link inherit" onclick={(event) => OPEN_PLATFORM[link.platform](event, link.handle)}>
            <span class="icon" title={LABELS_PLATFORM[link.platform]}>{@html ICONS_PLATFORM[link.platform]}</span>
            {link.handle}
          </button>
        {:else}
          None
        {/each}
      </dd>
      <dt>Created</dt>
      <dd>{created_at}</dd>
    </dl>
  </div>
</div>
