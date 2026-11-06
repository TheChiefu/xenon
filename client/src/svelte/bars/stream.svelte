<script lang="ts">
  import * as screens from "@/lib/rtc/screens.svelte";
  import * as users from "@/lib/users/data.svelte";
  import icon_x from "@/assets/icons/x.svg?raw";

  const watching: string | null = $derived(screens.getWatching());
</script>

<div class="row header-bar">
  <h3 class="grow">{watching === null ? "" : users.getDisplayName(watching)}</h3>

  <button
    class="icon-btn"
    aria-label="Close video"
    title="Close Video"
    disabled={watching === null}
    onclick={screens.stopWatching}
  >
    <span class="icon" aria-hidden="true">{@html icon_x}</span>
  </button>
</div>
