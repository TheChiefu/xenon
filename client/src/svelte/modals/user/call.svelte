<script lang="ts">
  import * as screen_share from "@/lib/client/screen_share.svelte";
  import { MSG_SAVED } from "@/lib/labels";

  // Screen Share
  let share_width = $state<number | null>(screen_share.getWidth());
  let share_height = $state<number | null>(screen_share.getHeight());
  let share_frame_rate = $state<number | null>(screen_share.getFrameRate());
  let share_bitrate = $state<number | null>(screen_share.getBitrate());
  let share_notify = $state("");

  function saveScreenShare() {
    if (share_width !== null) {
      screen_share.setWidth(Number(share_width));
    }
    if (share_height !== null) {
      screen_share.setHeight(Number(share_height));
    }
    if (share_frame_rate !== null) {
      screen_share.setFrameRate(Number(share_frame_rate));
    }
    if (share_bitrate !== null) {
      screen_share.setBitrate(Number(share_bitrate));
    }

    // Revert on invalid
    share_width = screen_share.getWidth();
    share_height = screen_share.getHeight();
    share_frame_rate = screen_share.getFrameRate();
    share_bitrate = screen_share.getBitrate();

    share_notify = MSG_SAVED;
  }
</script>

<style>
  .resolution {
    grid-template-columns: 1fr auto 1fr;
  }

  .resolution .x {
    align-self: center;
    color: var(--text-dim);
  }
</style>

<div class="section fields">
  <div class="grid-title">
    <h3>Screen Share</h3>
  </div>
  <p class="message">{share_notify}</p>

  <span>Resolution</span>
  <div class="resolution">
    <input
      id="share-width"
      name="share-width"
      type="number"
      min={screen_share.MIN_WIDTH}
      max={screen_share.MAX_WIDTH}
      step="any"
      aria-label="Width in pixels"
      bind:value={share_width}
    />
    <span class="x" aria-hidden="true">x</span>
    <input
      id="share-height"
      name="share-height"
      type="number"
      min={screen_share.MIN_HEIGHT}
      max={screen_share.MAX_HEIGHT}
      step="any"
      aria-label="Height in pixels"
      bind:value={share_height}
    />
  </div>

  <label for="share-frame-rate">Frame Rate: {share_frame_rate} FPS</label>
  <input
    id="share-frame-rate"
    name="share-frame-rate"
    type="range"
    min={screen_share.MIN_FRAME_RATE}
    max={screen_share.MAX_FRAME_RATE}
    step="1"
    bind:value={share_frame_rate}
  />

  <label for="share-bitrate">Bitrate: {share_bitrate} kbps</label>
  <input
    id="share-bitrate"
    name="share-bitrate"
    type="range"
    min={screen_share.MIN_BITRATE}
    max={screen_share.MAX_BITRATE}
    step="100"
    bind:value={share_bitrate}
  />

  <button class="full-row" onclick={saveScreenShare}>Save Changes</button>
</div>
