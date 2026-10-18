<script lang="ts">
    import * as menu from "@/lib/display/menu.svelte";
</script>

<style>

    .menu-backdrop {
        position: fixed;
        inset: 0;
        z-index: 10;
    }

    .menu {
        position: fixed;
        z-index: 11;
        display: flex;
        flex-direction: column;
        min-width: 10rem;
        border: 1px solid var(--border);
        background: var(--component);
    }

    button {
        text-align: left;
        white-space: nowrap;
    }

</style>

<!-- Right click context window -->
{#if menu.getPos()}
    <div class="menu-backdrop" role="presentation" onpointerdown={menu.close}></div>
    <div class="menu" style:left="{menu.getPos()?.x}px" style:top="{menu.getPos()?.y}px">
        {#each menu.getItems() as item}
            <button onclick={() => menu.chooseItem(item)}>{item.label}</button>
        {/each}
    </div>
{/if}
