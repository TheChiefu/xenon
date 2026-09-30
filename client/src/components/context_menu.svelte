<script lang="ts">
    import { chooseItem, closeMenu, getMenuItems, getMenuPos } from "@/lib/menu.svelte";
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
{#if getMenuPos()}
    <div class="menu-backdrop" role="presentation" onpointerdown={closeMenu}></div>
    <div class="menu" style:left="{getMenuPos()?.x}px" style:top="{getMenuPos()?.y}px">
        {#each getMenuItems() as item}
            <button onclick={() => chooseItem(item)}>{item.label}</button>
        {/each}
    </div>
{/if}
