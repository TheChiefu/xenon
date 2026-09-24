<script lang="ts">
  import { getActiveLogin } from "@/lib/session.svelte";
  import { getProfile } from "@/lib/profile.svelte";
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
  <button class="user" aria-label="Your profile">
    <span class="user-text">
      <span class="user-name">{getProfile()?.display_name ?? getActiveLogin()?.username}</span>
      <span class="user-handle">@{getActiveLogin()?.username}</span>
    </span>
  </button>
{/if}
