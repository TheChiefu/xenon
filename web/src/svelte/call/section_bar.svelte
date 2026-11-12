<script lang="ts">
  import type { Snippet } from "svelte";

  interface Props {
    label: string;
    detail: string;
    children?: Snippet;
  }

  let { label, detail, children }: Props = $props();
</script>

<style>

  .bar {
    flex: none;
    display: flex;
    align-items: center;
    gap: 0.5rem;
    min-height: 2.25rem;
    padding: 0.25rem 0.5rem;
    border-bottom: 1px solid var(--border);
    background: var(--component-raised);
  }

  .text {
    flex: 1;
    min-width: 0;
    display: flex;
    align-items: baseline;
    gap: 0.5rem;
  }

  .label {
    flex: none;
    color: var(--text-dim);
    font-size: 0.75rem;
    font-weight: bold;
    letter-spacing: 0.05em;
    text-transform: uppercase;
  }

  .detail {
    flex: 1;
    min-width: 0;
    white-space: nowrap;
    overflow: hidden;
    text-overflow: ellipsis;
  }

</style>

<div class="bar">
  <span class="text">
    <span class="label">{label}</span>
    <span class="detail">{detail}</span>
  </span>
  {@render children?.()}
</div>
