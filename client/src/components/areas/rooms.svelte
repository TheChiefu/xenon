<script lang="ts">
    import type { Folder } from "@/bindings/shared";
    import { Visibility } from "@/bindings/shared";
    import { LABELS_VISIBILITY } from "@/lib/labels";
    import { Section } from "@/lib/rooms/layout.svelte";
    import * as layout from "@/lib/rooms/layout.svelte";
    import type { RoomId } from "@/lib/rooms/data.svelte";
    import * as room from "@/lib/rooms/data.svelte";
    import { getSelectedRoom, selectRoom } from "@/lib/session.svelte";
    import * as drag from "@/lib/rooms/drag.svelte";
    import { Pane } from "@/lib/panes.svelte";
    import PaneArea from "@/components/pane.svelte";
    import * as menu from "@/lib/menu.svelte";
    import { folderCtx, paneCtx, roomCtx } from "@/lib/rooms/context_menu";
    import { nameFinished, isCreating, isRenaming, nameReset } from "@/lib/rooms/folders.svelte";
    import { close, isOpen } from "@/lib/rooms/creation.svelte";
    import NewRoom from "@/views/new_room.svelte";
    import icon_lock from "@/assets/icons/lock.svg?raw";
    import icon_lock_open from "@/assets/icons/lock-open.svg?raw";
    import icon_eye_off from "@/assets/icons/eye-off.svg?raw";
    import icon_folder from "@/assets/icons/folder.svg?raw";
    import icon_back from "@/assets/icons/back.svg?raw";
    import icon_arrow_down from "@/assets/icons/arrow-down.svg?raw";
    import icon_arrow_right from "@/assets/icons/arrow-right.svg?raw";

</script>

<style>

    /* Header */

    .header {
        display: flex;
        flex-direction: row;
        flex: none;
        padding: 0.5rem;
        align-items: center;
    }

    .header h3 {
        margin-right: auto;
        font-size: 1rem;
    }

    /* List */

    .list {
        --indent: 2rem;
        flex: 1;
        min-height: 0;
        display: flex;
        flex-direction: column;
        overflow-y: auto;
    }

    .list.moving {
        cursor: grabbing;
    }

    /* Rows */

    .list button {
        position: relative;
        margin-left: -1px;
        text-align: left;
        align-items: center;
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        touch-action: none;
        display: flex;
        justify-content: space-between;
    }

    .folder-name {
        flex: 1;
        margin-left: 0.5rem;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .selected {
        background-color: var(--component-hover);
    }

    .nested {
        padding-left: var(--indent);
    }

    .held {
        opacity: 0.75;
        background-color: var(--btn-action);
    }

    /* Drop Indicators */

    .drop-above::before,
    .drop-below::after {
        content: "";
        position: absolute;
        left: 0;
        right: 0;
        height: 2px;
        z-index: 1;
        pointer-events: none;
        background-color: var(--btn-action);
    }

    .drop-above::before {
        top: 0;
    }

    .drop-below::after {
        bottom: 0;
    }

    .nested.drop-above::before,
    .nested.drop-below::after,
    .open.drop-below::after {
        left: var(--indent);
    }

    .drop-into {
        outline: 2px solid var(--btn-action);
        outline-offset: -2px;
    }


</style>

<!-- Layout for movable room button -->
{#snippet roomButton(roomId: RoomId, nested: boolean)}
    {@const data = room.get(roomId)}
    {@const drop = drag.dropAt(roomId)}
    <button
        class:nested
        class:selected={getSelectedRoom() === roomId}
        class:held={drag.isHeld(roomId)}
        class:drop-above={drop === Section.Above}
        class:drop-below={drop === Section.Below}
        {@attach drag.linkRow(roomId)}
        onpointerdown={(event) => drag.start(event, roomId)}
        onclick={() => { if (layout.isLocked()) selectRoom(roomId); }}
        oncontextmenu={(event) => menu.open(event, roomCtx(roomId))}
    >
        {data?.name}
        {#if data?.visibility === Visibility.locked}
            <span class="icon" aria-hidden="true" title={LABELS_VISIBILITY[Visibility.locked]}>{@html icon_lock}</span>
        {:else if data?.visibility === Visibility.hidden}
            <span class="icon" aria-hidden="true" title={LABELS_VISIBILITY[Visibility.hidden]}>{@html icon_eye_off}</span>
        {/if}
    </button>
{/snippet}

<!-- Layout for movable folder -->
{#snippet folderButton(folder: Folder)}
    {#if isRenaming(folder)}
        <input
            type="text"
            autofocus
            value={folder.name}
            onkeydown={nameFinished}
            onblur={nameReset}
        />
    {:else}
        {@const drop = drag.dropAt(folder)}
        <button
            class:held={drag.isHeld(folder)}
            class:drop-above={drop === Section.Above}
            class:drop-below={drop === Section.Below}
            class:drop-into={drop === Section.Inside}
            class:open={!folder.collapsed}
            {@attach drag.row(folder)}
            onpointerdown={(event) => drag.start(event, folder)}
            onclick={() => layout.folderToggle(folder)}
            oncontextmenu={(event) => menu.open(event, folderCtx(folder))}
        >
            <span class="icon" aria-hidden="true" title="Folder">{@html icon_folder}</span>
            <span class="folder-name">{folder.name}</span>
            {#if folder.collapsed}
                <span class="icon" aria-hidden="true" title="Expand Folder">{@html icon_arrow_down}</span>
            {:else}
                <span class="icon" aria-hidden="true" title="Collapse Folder">{@html icon_arrow_right}</span>
            {/if}
        </button>
    {/if}
{/snippet}


<!-- Release can happen anywhere, not only over the list -->
<svelte:window onpointermove={drag.hold} onpointerup={drag.release} onpointercancel={drag.reset}/>

<!-- Room Pane -->
<PaneArea pane={Pane.room}>

    <!-- Display Header and related buttons -->
    <div class="header">
        <h3>Rooms</h3>
        {#if getSelectedRoom() !== null}
            <button
                class="icon-btn"
                aria-label="Close room"
                title="Close room"
                onclick={() => selectRoom(null)}
            >
                <span class="icon" aria-hidden="true">{@html icon_back}</span>
            </button>
        {/if}
        <button
            class="icon-btn"
            aria-label={layout.isLocked() ? "Unlock layout" : "Lock layout"}
            title={layout.isLocked() ? "Unlock layout" : "Lock layout"}
            onclick={layout.toggleLock}
        >
            <span class="icon" aria-hidden="true">{@html layout.isLocked() ? icon_lock : icon_lock_open}</span>
        </button>
    </div>

    <!-- Room List -->
    <div
        class="list"
        class:moving={drag.isActive()}
        oncontextmenu={(event) => menu.open(event, paneCtx())}
        role="presentation"
    >
        {#each layout.get() as entry}

            <!-- Room with no folder -->
            {#if typeof entry === "string"}
                {@render roomButton(entry, false)}
            {:else}

                <!-- Folder -->
                {@render folderButton(entry)}

                <!-- Room inside folder -->
                {#if !entry.collapsed}
                    {#each entry.rooms as roomId}
                        {@render roomButton(roomId, true)}
                    {/each}
                {/if}
            {/if}
        {/each}
    
        <!-- New Folder Input Bar -->
        {#if isCreating()}
            <input
                type="text"
                autofocus
                onkeydown={nameFinished}
                onblur={nameReset}
            />
        {/if}
    </div>

</PaneArea>

<!-- Room creation dialog, opened from the pane's context menu -->
{#if isOpen()}
    <NewRoom onClose={close}/>
{/if}
