<script lang="ts">
interface Props {
    src: string;
}

let { src }: Props = $props();
</script>

<style>

    .entry {
        display: block;
        width: 100%;
        max-width: 40rem;
        aspect-ratio: 16 / 9;
        margin-top: var(--row-gap);
        border: 1px solid var(--border);
    }

</style>

<iframe
    class="entry"
    {src}
    title="YouTube video player"
    allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share"
    referrerpolicy="strict-origin-when-cross-origin"
    allowfullscreen
></iframe>
