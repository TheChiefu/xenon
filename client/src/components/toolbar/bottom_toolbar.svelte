<script lang="ts">
  import PaneButtons from "@/components/toolbar/pane_buttons.svelte";
</script>

<style>
  .bottom-toolbar {
    display: flex;
    align-items: center;
    justify-content: space-around;
    height: 4rem;
    flex: none;
    border-top: 1px solid var(--border);
    background: var(--component);
  }
</style>

<nav class="bottom-toolbar">
  <PaneButtons/>
</nav>
