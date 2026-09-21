<script lang="ts">
  import Settings from "@/views/settings.svelte"
  import icon_refresh from "@/assets/icons/refresh.svg?raw";
  import icon_gear from "@/assets/icons/gear.svg?raw";
  import icon_headphones from "@/assets/icons/headphones.svg?raw";
  import icon_mic from "@/assets/icons/mic.svg?raw";

  let settingsOpen = $state(false);

  function openClientSettings() {
    settingsOpen = true;
  }

</script>

<div class="toolbar">

    <!-- Unimplemented - SKIP -->
    <button aria-label="Refresh Client">
        <span class="icon" aria-hidden="true">{@html icon_refresh}</span>
    </button>

    <button aria-label="Client Settings" onclick={openClientSettings}>
        <span class="icon" aria-hidden="true">{@html icon_gear}</span>
    </button>

    <!-- Unimplemented - SKIP -->
    <button aria-label="Toggle Speakers">
        <span class="icon" aria-hidden="true">{@html icon_headphones}</span>
    </button>

    <!-- Unimplemented - SKIP -->
    <button aria-label="Toggle Microphone">
        <span class="icon" aria-hidden="true">{@html icon_mic}</span>
    </button>

    {#if settingsOpen}
        <Settings onClose={() => settingsOpen = false}/>
    {/if}

</div>
