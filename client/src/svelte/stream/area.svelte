<script lang="ts">
  import * as screens from "@/lib/rtc/screens.svelte";

  // Plays the watched screen share, and follows it when another is watched
  function show(video: HTMLVideoElement): void {
    video.srcObject = screens.getWatchedScreen();
  }
</script>

<style>

  .video {
    flex: 1;
    min-height: 0;
    width: 100%;
    background: black;
  }

</style>

<video class="video" controls autoplay playsinline {@attach show}></video>
