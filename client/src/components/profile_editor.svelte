<script lang="ts">
  import ProfileCard from "@/components/profile_card.svelte";

  // Unsaved values, drawn on the preview as they are typed
  let display_name = $state("");
  let description = $state("");
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
</style>

<div class="editor">
  <div class="form">
    <label for="display-name">Display Name</label>
    <input id="display-name" placeholder="Name visible to others" bind:value={display_name}/>

    <label for="description">Description</label>
    <textarea id="description" bind:value={description}></textarea>

    <button>Save Changes</button>
  </div>

  <!-- Placeholder values until the profile is wired in -->
  <ProfileCard
    id="00000000000000000000000000000000"
    display_name={display_name}
    username="username"
    description={description}
    status="online"
    role="member"
    links={[{ platform: "xbox", handle: "My Gamertag" }, { platform: "steam", handle: "My Steam Name" }]}
    created_at="Jan 1, 2026"
    editable
  />
</div>
