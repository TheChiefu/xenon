<script lang="ts">
  import * as profile from "@/lib/profile.svelte";
  import * as users from "@/lib/users/data.svelte";
  import * as usersAPI from "@/lib/api/users";
  import * as filesAPI from "@/lib/api/files";
  import { failureText, type FetchedFile } from "@/lib/utils";
  import * as settings from "@/lib/settings.svelte";
  import icon_pencil from "@/assets/icons/pencil.svg?raw";
  import icon_x from "@/assets/icons/x.svg?raw";

  // Sent in place of a file id to clear a file on save
  const NIL_UUID = "00000000-0000-0000-0000-000000000000";

  // TEXT FIELDS //

  let display_name = $state(profile.get()?.display_name ?? "");
  let description = $state(profile.get()?.description ?? "");
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
  const user_id: string = $derived(profile.get()?.id ?? "");
  const avatar: FetchedFile | null = $derived(getProfileFile(avatar_preview, avatar_removed, users.getAvatar(user_id)));
  const banner: FetchedFile | null = $derived(getProfileFile(banner_preview, banner_removed, users.getBanner(user_id)));

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
      const stored = await filesAPI.upload(local);
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
      await usersAPI.updateProfile({
        display_name,
        description,
        avatar_file_id: avatar_saved,
        banner_file_id: banner_saved,
      });

      // Clear local files once uploaded
      avatar_file = null;
      banner_file = null;

      // Reload own profile and summary
      await profile.load();
    } catch (failure) {
      error = failureText(failure);
    }
  }
</script>

<style>
  .editor {
    display: flex;
    flex-direction: column;
    gap: var(--row-gap);
  }

  textarea {
    min-height: 10rem;
  }

  .row {
    --avatar-size: 3rem;
    position: relative;
    display: flex;
    align-items: center;
    align-self: center;
    width: 100%;
    max-width: 16rem;
    height: var(--avatar-size);
    background-color: var(--component);
    overflow: hidden;
    isolation: isolate;
    user-select: none;
    outline: 0.1rem ridge var(--border);
  }

  .avatar {
    position: relative;
    flex: none;
    width: var(--avatar-size);
    height: var(--avatar-size);
  }

  .banner, .picture {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .name {
    flex: 1;
    min-width: 0;
    padding: 0 var(--inline-gap);
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

  .error {
    color: var(--danger);
  }
</style>

<div class="editor">

  <!-- Member Row Preview -->
  <div class="row" {@attach settings.playOnHover}>
    {#if banner !== null}
      {#if banner.mime.startsWith("video/")}
        <video class="banner" src={banner.url} {@attach settings.animatePhoto}></video>
      {:else}
        <img class="banner" src={banner.url} alt=""/>
      {/if}
    {/if}

    <span class="avatar">
      {#if avatar !== null}
        {#if avatar.mime.startsWith("video/")}
          <video class="picture" src={avatar.url} {@attach settings.animatePhoto}></video>
        {:else}
          <img class="picture" src={avatar.url} alt=""/>
        {/if}
      {/if}
    </span>

    <span class="name text-outline">{display_name}</span>

  </div>

  <hr/>

  <div class="fields">
    <label for="display_name">Display Name</label>
    <input id="display_name" placeholder="Name visible to others" bind:value={display_name}/>

    <label for="description">Description</label>
    <textarea id="description" bind:value={description}></textarea>

    <span>Avatar</span>
    <div>
      <button onclick={() => avatar_input?.click()}>Change</button>
      <button onclick={removeAvatar} disabled={avatar === null}>Remove</button>
    </div>

    <span>Banner</span>
    <div>
      <button onclick={() => banner_input?.click()}>Change</button>
      <button onclick={removeBanner} disabled={banner === null}>Remove</button>
    </div>
  </div>


  <button onclick={save}>Save Changes</button>

  <input type="file" accept="image/*,video/*" bind:this={avatar_input} onchange={pickAvatar} hidden/>
  <input type="file" accept="image/*,video/*" bind:this={banner_input} onchange={pickBanner} hidden/>

  {#if error != ""}
    <p class="error">{error}</p>
  {/if}
</div>
