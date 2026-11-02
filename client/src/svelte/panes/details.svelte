<script lang="ts">
  import Members from "./members.svelte";
  import Call from "./call.svelte";

  interface Props {
    roomId: string | null;
    open: boolean;
  }

  let { roomId, open }: Props = $props();
</script>

<style>

  .half {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
  }

  .half + .half {
    border-top: 1px solid var(--border);
  }

</style>

<div class="half">
  <Members {roomId} {open}/>
</div>

<div class="half">
  <Call {roomId}/>
</div>
