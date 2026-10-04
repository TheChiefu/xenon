<script lang="ts">
    import type { Folder } from "@/bindings/shared";
    import { Visibility } from "@/bindings/types";
    import { LABELS_VISIBILITY } from "@/lib/labels";
    import { Section, folderToggle, getLayout, isLocked, isUnsaved, saveLayout } from "@/lib/rooms/layout.svelte";
    import { getRoomData, type RoomId, getSelectedRoom, selectRoom } from "@/lib/rooms/data.svelte";
    import { dragRelease,dragStart, dragStateReset, dragging, dropAt, isDragging, isHeld } from "@/lib/rooms/drag.svelte";
    import { resizeStop, getWidth, HANDLE_WIDTH, isResizing, reset, resize, resizeStart } from "@/lib/rooms/resize.svelte";
    import { openMenu } from "@/lib/menu.svelte";
    import { folderCtx, paneCtx, roomCtx } from "@/lib/rooms/context_menu";
    import { nameFinished, isCreating, isRenaming, nameReset } from "@/lib/rooms/folders.svelte";
    import { close, isOpen } from "@/lib/rooms/creation.svelte";
    import NewRoom from "@/views/new_room.svelte";
    import icon_save from "@/assets/icons/save.svg?raw";
    import icon_lock from "@/assets/icons/lock.svg?raw";
    import icon_lock_open from "@/assets/icons/lock-open.svg?raw";
    import icon_eye_off from "@/assets/icons/eye-off.svg?raw";
    import icon_folder from "@/assets/icons/folder.svg?raw";
    import icon_arrow_down from "@/assets/icons/arrow-down.svg?raw";
    import icon_arrow_right from "@/assets/icons/arrow-right.svg?raw";

    let pane: HTMLDivElement | undefined = $state();

</script>

<style>

    /* Pane */

    .rooms {
        --indent: 2rem;
        position: relative;
        display: flex;
        flex-direction: column;
        flex: none;
        border-right: 1px solid var(--border);
        background: rgba(30,30,30, 0.5);
        overflow: hidden;
    }

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
        flex: 1;
        display: flex;
        flex-direction: column;
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

    /* Resize Handle */

    .resize-handle {
        position: absolute;
        top: 0;
        right: calc(var(--handle-width) / -2);
        width: var(--handle-width);
        height: 100%;
        z-index: 1;
        touch-action: none;
        background-color: var(--border);
        cursor: col-resize;
    }

    .resize-handle.dragging {
        background-color: var(--btn-action);
    }

</style>

<!-- Layout for movable room button -->
{#snippet roomButton(roomId: RoomId, nested: boolean)}
    {@const room = getRoomData(roomId)}
    {@const drop = dropAt(roomId)}
    <button
        class:nested
        class:selected={getSelectedRoom() === roomId}
        class:held={isHeld(roomId)}
        class:drop-above={drop === Section.Above}
        class:drop-below={drop === Section.Below}
        onpointerdown={(event) => dragStart(event, roomId)}
        onpointermove={(event) => dragging(event, roomId)}
        onclick={() => { if (isLocked()) selectRoom(roomId); }}
        oncontextmenu={(event) => openMenu(event, roomCtx(roomId))}
    >
        {room?.name}
        {#if room?.visibility === Visibility.Locked}
            <span class="icon" aria-hidden="true" title={LABELS_VISIBILITY[Visibility.Locked]}>{@html icon_lock}</span>
        {:else if room?.visibility === Visibility.Hidden}
            <span class="icon" aria-hidden="true" title={LABELS_VISIBILITY[Visibility.Hidden]}>{@html icon_eye_off}</span>
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
        {@const drop = dropAt(folder)}
        <button
            class:held={isHeld(folder)}
            class:drop-above={drop === Section.Above}
            class:drop-below={drop === Section.Below}
            class:drop-into={drop === Section.Inside}
            class:open={!folder.collapsed}
            onpointerdown={(event) => dragStart(event, folder)}
            onpointermove={(event) => dragging(event, folder)}
            onclick={() => folderToggle(folder)}
            oncontextmenu={(event) => openMenu(event, folderCtx(folder))}
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
<svelte:window onpointerup={dragRelease} onpointercancel={dragStateReset}/>

<!-- Room Pane -->
<div
    class="rooms"
    style="width: {getWidth()}px"
    style:--handle-width="{HANDLE_WIDTH}px"
    oncontextmenu={(event) => openMenu(event, paneCtx())}
    role="presentation"
    bind:this={pane}
>

    <!-- Display Header and related buttons -->
    <div class="header">
        <h3>Rooms</h3>
        <button
            class="icon-btn"
            style:visibility={isUnsaved() ? "visible" : "hidden"}
            aria-label="Save layout"
            title="Save layout"
            onclick={saveLayout}
        >
            <span class="icon" aria-hidden="true">{@html icon_save}</span>
        </button>
        {#if !isLocked()}
            <span class="icon" aria-hidden="true">{@html icon_lock_open}</span>
        {/if}
    </div>

    <!-- Room List -->
    <div
        class="list"
        class:moving={isDragging()}
        role="presentation"
    >
        {#each getLayout() as entry}

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

    <!-- Vertical Bar for resizing room pane-->
    <div
        class="resize-handle"
        class:dragging={isResizing()}
        role="separator"
        aria-orientation="vertical"
        aria-label="Resize room list"
        onpointerdown={resizeStart}
        onpointermove={(event) => resize(event, pane)}
        onpointerup={resizeStop}
        onlostpointercapture={resizeStop}
        ondblclick={reset}
    ></div>
</div>

<!-- Room creation dialog, opened from the pane's context menu -->
{#if isOpen()}
    <NewRoom onClose={close}/>
{/if}
