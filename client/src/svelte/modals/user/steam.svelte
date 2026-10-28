<script lang="ts">
  import { LABELS_PLATFORM, ICONS_PLATFORM } from "@/lib/labels";
  import { Platform } from "@/bindings/types/platforms";
  import icon_link from "@/assets/icons/link.svg?raw";
  import icon_link_broken from "@/assets/icons/link_broken.svg?raw";

  let account_linked_steam = $state(false);
  let account_steam_user = $state("");
</script>

<div class="section platform">
    <div class="platform-row">
      <span class="icon" title={LABELS_PLATFORM[Platform.steam]}>{@html ICONS_PLATFORM[Platform.steam]}</span>
      <p class="title">Steam</p>
    </div>

    <div class="platform-row">
      <div class="platform-text">
        <p class="handle">{account_linked_steam ? account_steam_user : "Not Linked"}</p>
      </div>

      <div class="platform-buttons">
        <button
          class="icon-btn {account_linked_steam ? 'is-danger' : ''}"
          aria-label={account_linked_steam ? "Unlink Steam" : "Link Steam"}
          disabled
        >
          <span class="icon" aria-hidden="true">{@html account_linked_steam ? icon_link_broken : icon_link}</span>
        </button>
      </div>
    </div>
</div>
