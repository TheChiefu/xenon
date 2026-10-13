<script lang="ts">
  import { LABELS_PLATFORM, ICONS_PLATFORM } from "@/lib/labels";
  import { Platform } from "@/bindings/shared";
  import icon_link from "@/assets/icons/link.svg?raw";
  import icon_link_broken from "@/assets/icons/link_broken.svg?raw";

  let account_linked_steam = $state(false);
  let account_steam_user = $state("");
</script>

<style>
  .platform {
    grid-template-columns: auto 1fr auto;
  }

  .handle {
    font-style: italic;
    color: var(--text-dim);
    cursor: default;
    user-select: none;
  }
</style>

<div class="section rows platform">
    <h3 class="full-row">Steam</h3>

    <span class="icon" title={LABELS_PLATFORM[Platform.steam]}>{@html ICONS_PLATFORM[Platform.steam]}</span>
    <p class="handle">{account_linked_steam ? account_steam_user : "Not Linked"}</p>
    <button
      class="icon-btn {account_linked_steam ? 'is-danger' : ''}"
      aria-label={account_linked_steam ? "Unlink Steam" : "Link Steam"}
      disabled
    >
      <span class="icon" aria-hidden="true">{@html account_linked_steam ? icon_link_broken : icon_link}</span>
    </button>
</div>
