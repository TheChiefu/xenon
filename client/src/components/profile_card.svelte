<script lang="ts">
  import type { LinkedAccount } from "@/bindings/shared";
  import type { GlobalRole, Status } from "@/bindings/shared";
  import { ICONS_PLATFORM, LABELS_PLATFORM, LABELS_ROLE, LABELS_STATUS } from "@/lib/labels";
  import icon_pencil from "@/assets/icons/pencil.svg?raw";

  interface Props {
    id: string;
    display_name: string;
    username: string;
    description: string;
    status: Status;
    role: GlobalRole;
    links: LinkedAccount[];
    created_at: string;
    deleted_at?: string | null;
    avatar_url?: string | null;
    banner_url?: string | null;

    // Shows the edit buttons on the banner and avatar
    editable?: boolean;
  }
  let {
    id,
    display_name,
    username,
    description,
    status,
    role,
    links,
    created_at,
    deleted_at = null,
    avatar_url = null,
    banner_url = null,
    editable = false,
  }: Props = $props();

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
    height: 6rem;
    background: var(--border) center / cover no-repeat;
  }

  .avatar {
    position: absolute;
    left: 1rem;
    bottom: -2.5rem;
    width: 5rem;
    height: 5rem;
    border: 3px solid var(--component);
    border-radius: 50%;
    background: var(--background) center / cover no-repeat;
  }

  .edit-banner {
    position: absolute;
    top: 0.5rem;
    right: 0.5rem;
  }

  .edit-avatar {
    position: absolute;
    right: -0.25rem;
    bottom: -0.25rem;
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
  <div class="banner" style:background-image={banner_url ? `url("${banner_url}")` : null}>
    {#if editable}
      <button class="icon-btn edit-banner" aria-label="Edit banner">
        <span class="icon" aria-hidden="true">{@html icon_pencil}</span>
      </button>
    {/if}

    <div class="avatar" style:background-image={avatar_url ? `url("${avatar_url}")` : null}>
      {#if editable}
        <button class="icon-btn edit-avatar" aria-label="Edit avatar">
          <span class="icon" aria-hidden="true">{@html icon_pencil}</span>
        </button>
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
      <dt>Status</dt>
      <dd>{LABELS_STATUS[status]}</dd>
      <dt>Role</dt>
      <dd>{LABELS_ROLE[role]}</dd>
      <dt>Linked</dt>
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
