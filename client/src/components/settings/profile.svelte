<script lang="ts">
  import { getProfile, loadProfile } from "@/lib/profile.svelte";
  import { getAvatarOfUser, getBannerOfUser, loadProfileFilesOfUser } from "@/lib/users.svelte";
  import { updateProfile } from "@/lib/api/users";
  import { uploadFile } from "@/lib/api/files";
  import { failureText, type FetchedFile } from "@/lib/utils";
  import { animatePhoto, playOnHover } from "@/lib/settings.svelte";
  import icon_pencil from "@/assets/icons/pencil.svg?raw";
  import icon_x from "@/assets/icons/x.svg?raw";

  // Sent in place of a file id to clear a file on save
  const NIL_UUID = "00000000-0000-0000-0000-000000000000";

  // TEXT FIELDS //

  let display_name = $state(getProfile()?.display_name ?? "");
  let description = $state(getProfile()?.description ?? "");
  let error = $state("");

  // FILES //

  // File inputs for selecting local files
  let avatar_input = $state<HTMLInputElement | null>(null);
  let banner_input = $state<HTMLInputElement | null>(null);

  // Local files waiting for upload on save
  let avatar_file: File | null = null;
  let banner_file: File | null = null;

  // Previews of local files
  let avatar_preview = $state<FetchedFile | null>(null);
  let banner_preview = $state<FetchedFile | null>(null);

  // Marked for removal on save, cleared when a local file is set
  let avatar_removed = $state(false);
  let banner_removed = $state(false);

  // Avatar and banner, counting unsaved changes
  const user_id = getProfile()?.id ?? "";
  const avatar: FetchedFile | null = $derived(getProfileFile(avatar_preview, avatar_removed, getAvatarOfUser(user_id)));
  const banner: FetchedFile | null = $derived(getProfileFile(banner_preview, banner_removed, getBannerOfUser(user_id)));

  // Load stored avatar and banner of own user
  $effect(() => {
    loadProfileFilesOfUser(user_id);
  });

  // HELPERS //

  // Gets a profile file, counting unsaved changes
  // - local: Local file
  // - removed: Whether a file is marked for removal
  // - stored: File from a user file map
  function getProfileFile(local: FetchedFile | null, removed: boolean, stored: FetchedFile | undefined): FetchedFile | null {

    // Return nothing for a removed file
    if (removed) {
      return null;
    }

    // Return a local file over a stored one
    if (local !== null) {
      return local;
    }

    // Return nothing when no file is stored
    if (stored === undefined) {
      return null;
    }

    return stored;
  }

  // Revokes a preview's URL
  function revokePreview(file: FetchedFile | null): void {
    if (file !== null) {
      URL.revokeObjectURL(file.url);
    }
  }

  // Gets a file change to send on save
  // - local: Local file
  // - removed: Whether a file is marked for removal
  async function getFileChange(local: File | null, removed: boolean): Promise<string | null> {

    // Upload a local file for a new id
    if (local !== null) {
      const stored = await uploadFile(local);
      return stored.id;
    }

    // Send a zeroed id to remove a file
    if (removed) {
      return NIL_UUID;
    }

    // Send no id to leave a file unchanged
    return null;
  }

  // ACTIONS //

  function pickAvatar() {
    const file = avatar_input?.files?.[0];
    if (file !== undefined) {
      revokePreview(avatar_preview);
      avatar_file = file;
      avatar_preview = { url: URL.createObjectURL(file), mime: file.type };
      avatar_removed = false;
    }
  }

  function pickBanner() {
    const file = banner_input?.files?.[0];
    if (file !== undefined) {
      revokePreview(banner_preview);
      banner_file = file;
      banner_preview = { url: URL.createObjectURL(file), mime: file.type };
      banner_removed = false;
    }
  }

  function removeAvatar() {
    revokePreview(avatar_preview);
    avatar_file = null;
    avatar_preview = null;
    avatar_removed = true;
  }

  function removeBanner() {
    revokePreview(banner_preview);
    banner_file = null;
    banner_preview = null;
    banner_removed = true;
  }

  // Saves profile changes
  async function save() {

    // Clear a previous error
    error = "";

    try {

      // Get file changes to send
      const avatar_saved = await getFileChange(avatar_file, avatar_removed);
      const banner_saved = await getFileChange(banner_file, banner_removed);

      // Send profile changes
      await updateProfile({
        display_name,
        description,
        avatar_file_id: avatar_saved,
        banner_file_id: banner_saved,
      });

      // Clear local files once uploaded
      avatar_file = null;
      banner_file = null;

      // Reload own profile and summary
      await loadProfile();
    } catch (failure) {
      error = failureText(failure);
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
