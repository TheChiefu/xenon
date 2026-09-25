<script lang="ts">
  import { getUrl, getToken, getActiveLogin } from "@/lib/session.svelte";
  import { getProfile } from "@/lib/profile.svelte";
  import { fetchFileBlob, type FetchedFile } from "@/lib/utils";

  let avatar = $state<FetchedFile | null>(null);
  let profileOpen = $state(false);

  $effect(() => {
    const url = getUrl();
    const token = getToken();
    const fileId = getProfile()?.avatar_file_id;
    if (url === null || token === null || fileId == null) {
      avatar = null;
      return;
    }

    fetchFileBlob(url, fileId, token).then((file) => { avatar = file; });

    return () => {
      if (avatar) URL.revokeObjectURL(avatar.url);
    };
  });
</script>

<style>
  .user {
    display: flex;
    align-items: center;
    gap: 0.625rem;
    height: 100%;
    padding: 0 1.125rem;
    border: 0;
    border-left: 1px solid var(--border);
    background: var(--component);
    color: var(--text);
    font: inherit;
    text-align: left;
    cursor: pointer;
  }

  .user-avatar {
    width: 2rem;
    height: 2rem;
    flex: none;
    border-radius: 50%;
    object-fit: cover;
  }

  .user-text {
    display: grid;
    min-width: 0;
  }

  .user-name, .user-handle {
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .user-name {
    font-weight: bold;
  }

  .user-handle {
    color: var(--text-dim);
    font-size: 0.78rem;
  }
</style>

{#if getActiveLogin() !== null}
  <button class="user" aria-label="Your profile" onclick={() => profileOpen = true}>
    {#if avatar?.mime.startsWith("video/")}
      <video class="user-avatar" src={avatar.url} autoplay muted loop playsinline></video>
    {:else if avatar}
      <img class="user-avatar" src={avatar.url} alt=""/>
    {/if}
    <span class="user-text">
      <span class="user-name">{getProfile()?.display_name ?? getActiveLogin()?.username}</span>
      <span class="user-handle">@{getActiveLogin()?.username}</span>
    </span>
  </button>
{/if}
