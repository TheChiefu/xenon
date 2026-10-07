<script lang="ts">
  import type { LinkedAccount } from "@/bindings/shared";
  import type { GlobalRole, Status } from "@/bindings/shared";
  import { Platform } from "@/bindings/shared";
  import type { Device, GameActivity } from "@/bindings/sockets/events";
  import { ICONS_PLATFORM, LABELS_DEVICE, LABELS_PLATFORM, LABELS_ROLE, LABELS_STATUS } from "@/lib/labels";
  import { fetchFileBlob, type FetchedFile } from "@/lib/utils";
  import { animatePhoto } from "@/lib/settings.svelte";

  interface Props {
    id: string;
    display_name: string;
    username: string;
    description: string;
    status?: Status | null;
    device?: Device | null;
    game?: GameActivity | null;
    role: GlobalRole;
    links: LinkedAccount[];
    created_at: string;
    deleted_at?: string | null;
    avatar_file_id?: string | null;
    banner_file_id?: string | null;
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
    deleted_at = null,
    avatar_file_id = null,
    banner_file_id = null,
  }: Props = $props();

  let avatar = $state<FetchedFile | null>(null);
  let banner = $state<FetchedFile | null>(null);

  loadPictures();

  async function loadPictures() {
    if (avatar_file_id !== null) {
      avatar = await fetchFileBlob(avatar_file_id);
    }

    if (banner_file_id !== null) {
      banner = await fetchFileBlob(banner_file_id);
    }
  }
</script>

<style>
  .card {
    width: 18rem;
    border: 1px solid var(--border);
    background: var(--component);
    color: var(--text);
    overflow: hidden;
  }

  .banner {
    position: relative;
    height: 3rem;
    background: var(--border);
  }

  .avatar {
    position: absolute;
    left: 1rem;
    bottom: -2.5rem;
    width: 5rem;
    height: 5rem;
    border: 3px solid var(--component);
    background: var(--background);
  }

  .media {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .body {
    padding: 3rem 1rem 1rem;
  }

  .name {
    font-size: 1.25rem;
    font-weight: bold;
  }

  .handle, .id {
    color: var(--text-dim);
    font-size: 0.85rem;
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

  .id {
    overflow-wrap: anywhere;
  }

  .meta {
    display: grid;
    grid-template-columns: auto 1fr;
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

  .id {
    font-style: italic;
    font-family: monospace;
  }

  .link {
    display: flex;
    align-items: center;
    gap: 0.375rem;
  }
</style>

<div class="card">
  <div class="banner">
    {#if banner !== null}
      {#if banner.mime.startsWith("video/")}
        <video class="media" src={banner.url} {@attach animatePhoto} muted loop playsinline></video>
      {:else}
        <img class="media" src={banner.url} alt=""/>
      {/if}
    {/if}

    <div class="avatar">
      {#if avatar !== null}
        {#if avatar.mime.startsWith("video/")}
          <video class="media" src={avatar.url} {@attach animatePhoto} muted loop playsinline></video>
        {:else}
          <img class="media" src={avatar.url} alt=""/>
        {/if}
      {/if}
    </div>
  </div>

  <div class="body">
    <p class="name">{display_name || "Display Name"}</p>
    <p class="handle">@{username}</p>
    <p class="id">#{id}</p>

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
        <dd class="link">
          <span class="icon" title={LABELS_PLATFORM[game.platform]}>{@html ICONS_PLATFORM[game.platform]}</span>
          {game.title ?? game.activity ?? game.status}
        </dd>
      {/if}

      <dt>Global Role</dt>
      <dd>{LABELS_ROLE[role]}</dd>
      <dt>Linked Accounts</dt>
      <dd>
        {#each links as link}
          <p class="link">
            <span class="icon" title={LABELS_PLATFORM[link.platform]}>{@html ICONS_PLATFORM[link.platform]}</span>
            {link.handle}
          </p>
        {:else}
          None
        {/each}
      </dd>
      <dt>Created</dt>
      <dd>{created_at}</dd>
      {#if deleted_at}
        <dt>Deleted</dt>
        <dd>{deleted_at}</dd>
      {/if}
    </dl>
  </div>
</div>
