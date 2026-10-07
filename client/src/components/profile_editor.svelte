<script lang="ts">
  import { getProfile, loadProfile } from "@/lib/profile.svelte";
  import { updateProfile } from "@/lib/api/users";
  import { uploadFile } from "@/lib/api/files";
  import { fetchFileBlob, type FetchedFile } from "@/lib/utils";
  import { animatePhoto } from "@/lib/settings.svelte";
  import icon_pencil from "@/assets/icons/pencil.svg?raw";

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

  async function save() {
    error = "";

    try {
      let avatar_saved: string | null = null;
      if (avatar_file !== null) {
        const stored = await uploadFile(avatar_file);
        avatar_saved = stored.id;
      }

      let banner_saved: string | null = null;
      if (banner_file !== null) {
        const stored = await uploadFile(banner_file);
        banner_saved = stored.id;
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
    gap: 1.5rem;
    align-items: flex-start;
  }

  .form {
    display: flex;
    flex-direction: column;
    flex: 1;
    gap: 0.5rem;
  }

  textarea {
    min-height: 10rem;
  }

  .row {
    --avatar-size: 3rem;
    position: relative;
    width: 18rem;
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
    padding: 0;
    display: grid;
    place-items: center;
    border: 0;
    background: transparent;
    color: inherit;
  }

  .banner {
    position: absolute;
    inset: 0;
    z-index: -1;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .avatar .icon {
    position: relative;
    z-index: 1;
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
    border: 0;
    background: transparent;
    color: inherit;
  }

  .error {
    color: var(--danger);
  }
</style>

<div class="editor">
  <div class="form">

    <!-- Member Row Preview -->
    <div class="row">
      {#if banner !== null}
        {#if banner.mime.startsWith("video/")}
          <video class="banner" src={banner.url} {@attach animatePhoto} muted loop playsinline></video>
        {:else}
          <img class="banner" src={banner.url} alt=""/>
        {/if}
      {/if}

      <button class="avatar" aria-label="Change avatar" onclick={() => avatar_input?.click()}>
        {#if avatar !== null}
          {#if avatar.mime.startsWith("video/")}
            <video class="picture" src={avatar.url} {@attach animatePhoto} muted loop playsinline></video>
          {:else}
            <img class="picture" src={avatar.url} alt=""/>
          {/if}
        {/if}

        <span class="icon" aria-hidden="true">{@html icon_pencil}</span>
      </button>

      <input class="name text-outline" placeholder="Name visible to others" bind:value={display_name}/>

      <button class="icon-btn" aria-label="Change banner" onclick={() => banner_input?.click()}>
        <span class="icon" aria-hidden="true">{@html icon_pencil}</span>
      </button>
    </div>

    <label for="description">Description</label>
    <textarea id="description" bind:value={description}></textarea>

    <input type="file" accept="image/*,video/*" bind:this={avatar_input} onchange={pickAvatar} hidden/>
    <input type="file" accept="image/*,video/*" bind:this={banner_input} onchange={pickBanner} hidden/>

    <button onclick={save}>Save Changes</button>

    {#if error != ""}
      <p class="error">{error}</p>
    {/if}
  </div>
</div>
