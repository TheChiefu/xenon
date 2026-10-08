<script lang="ts">
  import { getProfile, loadProfile } from "@/lib/profile.svelte";
  import { updateProfile } from "@/lib/api/users";
  import { uploadFile } from "@/lib/api/files";
  import { fetchFileBlob, type FetchedFile } from "@/lib/utils";
  import { animatePhoto, playOnHover } from "@/lib/settings.svelte";
  import icon_pencil from "@/assets/icons/pencil.svg?raw";
  import icon_x from "@/assets/icons/x.svg?raw";

  const NIL_UUID = "00000000-0000-0000-0000-000000000000";

  let display_name = $state(getProfile()?.display_name ?? "");
  let description = $state(getProfile()?.description ?? "");
  let error = $state("");
  let avatar_input = $state<HTMLInputElement | null>(null);
  let banner_input = $state<HTMLInputElement | null>(null);
  let avatar_file: File | null = null;
  let banner_file: File | null = null;
  let avatar = $state<FetchedFile | null>(null);
  let banner = $state<FetchedFile | null>(null);

  loadPictures();

  async function loadPictures() {
    const stored = getProfile();
    if (stored === null) {
      return;
    }

    if (stored.avatar_file_id !== null) {
      avatar = await fetchFileBlob(stored.avatar_file_id);
    }

    if (stored.banner_file_id !== null) {
      banner = await fetchFileBlob(stored.banner_file_id);
    }
  }

  function pickAvatar() {
    const file = avatar_input?.files?.[0];
    if (file !== undefined) {
      avatar_file = file;
      avatar = { url: URL.createObjectURL(file), mime: file.type };
    }
  }

  function pickBanner() {
    const file = banner_input?.files?.[0];
    if (file !== undefined) {
      banner_file = file;
      banner = { url: URL.createObjectURL(file), mime: file.type };
    }
  }

  function removeAvatar() {
    avatar_file = null;
    avatar = null;
  }

  function removeBanner() {
    banner_file = null;
    banner = null;
  }

  async function save() {
    error = "";

    try {
      let avatar_saved: string | null = null;
      if (avatar_file !== null) {
        const stored = await uploadFile(avatar_file);
        avatar_saved = stored.id;
      } else if (avatar === null) {
        avatar_saved = NIL_UUID;
      }

      let banner_saved: string | null = null;
      if (banner_file !== null) {
        const stored = await uploadFile(banner_file);
        banner_saved = stored.id;
      } else if (banner === null) {
        banner_saved = NIL_UUID;
      }

      await updateProfile({
        display_name,
        description,
        avatar_file_id: avatar_saved,
        banner_file_id: banner_saved,
      });

      avatar_file = null;
      banner_file = null;

      await loadProfile();
    } catch (failure) {
      error = failure instanceof Error ? failure.message : String(failure);
    }
  }
</script>

<style>
  .editor {
    display: flex;
    flex-direction: column;
    gap: 0.5rem;
  }

  .columns {
    display: flex;
    gap: 1.5rem;
    align-items: flex-start;
  }

  .controls {
    display: flex;
    flex-direction: column;
    flex: 1;
    gap: 0.5rem;
  }

  .preview {
    display: flex;
    flex-direction: column;
    flex: none;
    width: 18rem;
    gap: 0.5rem;
    cursor: default;
    user-select: none;
    -webkit-user-select: none;
  }

  .buttons {
    display: flex;
    align-items: center;
    gap: 0.5rem;
  }

  .buttons span:first-child {
    flex: 1;
  }

  textarea {
    min-height: 10rem;
  }

  .row {
    --avatar-size: 3rem;
    position: relative;
    height: var(--avatar-size);
    display: flex;
    align-items: center;
    background-color: var(--component);
    overflow: hidden;
    isolation: isolate;
  }

  .avatar {
    position: relative;
    width: var(--avatar-size);
    height: var(--avatar-size);
    flex: none;
  }

  .banner {
    position: absolute;
    inset: 0;
    z-index: -1;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .picture {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .name {
    flex: 1;
    min-width: 0;
    padding: 0 0.5rem;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .error {
    color: var(--danger);
  }
</style>

<div class="editor">
  <div class="columns">
    <div class="controls">
      <label for="display_name">Display Name</label>
      <input id="display_name" placeholder="Name visible to others" bind:value={display_name}/>

      <label for="description">Description</label>
      <textarea id="description" bind:value={description}></textarea>
    </div>

    <div class="preview">
      <!-- Member Row Preview -->
      <div class="row" {@attach playOnHover}>
        {#if banner !== null}
          {#if banner.mime.startsWith("video/")}
            <video class="banner" src={banner.url} {@attach animatePhoto}></video>
          {:else}
            <img class="banner" src={banner.url} alt=""/>
          {/if}
        {/if}

        <span class="avatar">
          {#if avatar !== null}
            {#if avatar.mime.startsWith("video/")}
              <video class="picture" src={avatar.url} {@attach animatePhoto}></video>
            {:else}
              <img class="picture" src={avatar.url} alt=""/>
            {/if}
          {/if}
        </span>

        <span class="name text-outline">{display_name}</span>
      </div>

      <div class="buttons">
        <span>Avatar</span>
        <button class="icon-btn" aria-label="Change Avatar" title="Change Avatar" onclick={() => avatar_input?.click()}>
          <span class="icon" aria-hidden="true">{@html icon_pencil}</span>
        </button>
        <button class="icon-btn" aria-label="Remove Avatar" title="Remove Avatar" onclick={removeAvatar} disabled={avatar === null}>
          <span class="icon" aria-hidden="true">{@html icon_x}</span>
        </button>
      </div>

      <div class="buttons">
        <span>Banner</span>
        <button class="icon-btn" aria-label="Change Banner" title="Change Banner" onclick={() => banner_input?.click()}>
          <span class="icon" aria-hidden="true">{@html icon_pencil}</span>
        </button>
        <button class="icon-btn" aria-label="Remove Banner" title="Remove Banner" onclick={removeBanner} disabled={banner === null}>
          <span class="icon" aria-hidden="true">{@html icon_x}</span>
        </button>
      </div>
    </div>
  </div>

  <input type="file" accept="image/*,video/*" bind:this={avatar_input} onchange={pickAvatar} hidden/>
  <input type="file" accept="image/*,video/*" bind:this={banner_input} onchange={pickBanner} hidden/>

  <button onclick={save}>Save Changes</button>

  {#if error != ""}
    <p class="error">{error}</p>
  {/if}
</div>
