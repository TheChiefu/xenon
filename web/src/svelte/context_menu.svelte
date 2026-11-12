<script lang="ts">
    import * as menu from "@/lib/display/menu.svelte";

    let contextMenu = $state<HTMLDialogElement | null>(null);

    $effect(() => {
        contextMenu?.showModal();
    });

    function pressAway(event: PointerEvent) {
        if (event.target === contextMenu) {
            menu.close();
        }
    }

    function cancel(event: Event) {
        event.preventDefault();
        menu.close();
    }
</script>

<style>

    .context {
        inset: auto;
        margin: 0;
        padding: 0;
        min-width: 10rem;
        border: 1px solid var(--border);
        background: var(--component);
        color: var(--text);
    }

    .context[open] {
        display: flex;
        flex-direction: column;
    }

    .context::backdrop {
        background: none;
    }

    button {
        text-align: left;
        white-space: nowrap;
    }

</style>

<!-- Right click context window -->
{#if menu.getPos()}
    <dialog
        class="context"
        bind:this={contextMenu}
        oncancel={cancel}
        onpointerdown={pressAway}
        style:left="{menu.getPos()?.x}px"
        style:top="{menu.getPos()?.y}px"
    >
        {#each menu.getItems() as item}
            <button onclick={() => menu.chooseItem(item)}>{item.label}</button>
        {/each}
    </dialog>
{/if}
