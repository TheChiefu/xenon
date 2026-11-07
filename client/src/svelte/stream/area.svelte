<script lang="ts">
  import * as screens from "@/lib/rtc/screens.svelte";

  let video = $state<HTMLVideoElement | null>(null);

  // The watched stream's actual size and framerate, measured from the video
  let width = $state(0);
  let height = $state(0);
  let fps = $state(0);

  // Frames counted since the last measurement, and the media time it started at
  let frames = 0;
  let window_start = 0;
  let frame_handle = 0;

  // Plays the watched screen share, and follows it when another is watched
  $effect(() => {
    const stream = screens.getWatchedScreen();
    if (video === null) {
      return;
    }

    video.srcObject = stream;

    video.cancelVideoFrameCallback(frame_handle);
    frames = 0;
    window_start = 0;

    if (stream === null) {
      width = 0;
      height = 0;
      fps = 0;
      return;
    }

    if (typeof video.requestVideoFrameCallback !== "function") {
      return;
    }

    frame_handle = video.requestVideoFrameCallback(onFrame);
  });

  // Counts the stream's frames, reporting its size and framerate each second
  function onFrame(now: number, metadata: VideoFrameCallbackMetadata): void {
    if (video === null) {
      return;
    }

    width = video.videoWidth;
    height = video.videoHeight;
    frames += 1;

    const elapsed = metadata.mediaTime - window_start;
    if (elapsed >= 1) {
      fps = Math.round((frames / elapsed) * 10) / 10;
      frames = 0;
      window_start = metadata.mediaTime;
    }

    frame_handle = video.requestVideoFrameCallback(onFrame);
  }
</script>

<style>

  .stage {
    position: relative;
    flex: 1;
    min-height: 0;
  }

  .video {
    width: 100%;
    height: 100%;
    background: black;
  }

  .stats {
    position: absolute;
    top: 0.5rem;
    left: 0.5rem;
    color: rgba(255, 255, 255, 0.45);
    font-size: 0.75rem;
    font-variant-numeric: tabular-nums;
    pointer-events: none;
    user-select: none;
  }

</style>

<div class="stage">
  <video class="video" bind:this={video} controls autoplay playsinline></video>

  {#if width > 0}
    <span class="stats">{width}×{height} · {fps} fps</span>
  {/if}
</div>
