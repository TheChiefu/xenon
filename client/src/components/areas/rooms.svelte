<script lang="ts">
    import type { Folder } from "@/bindings/shared";
    import { Drop, getLayout, isLocked, toggleFolder } from "@/lib/rooms/layout.svelte";
    import { getRoomData } from "@/lib/rooms/data.svelte";
    import { getSelectedRoom, selectRoom } from "@/lib/rooms/selection.svelte";
    import { cancelDrag, dropAt, endDrag, hover, hoverEnd, isDragging, isHeld, startDrag } from "@/lib/rooms/drag.svelte";
    import { endResize, getWidth, HANDLE_WIDTH, isResizing, resetWidth, resize, startResize } from "@/lib/rooms/resize.svelte";
    import { openMenu } from "@/lib/menu.svelte";
    import { folderCtx, paneCtx, roomCtx } from "@/lib/rooms/context_menu";
    import { finishCreatingFolder, finishRenamingFolder, isCreatingFolder, isRenamingFolder, stopCreatingFolder, stopRenamingFolder } from "@/lib/rooms/folders.svelte";
    import icon_lock from "@/assets/icons/lock.svg?raw";
    import icon_lock_open from "@/assets/icons/lock-open.svg?raw";
    import icon_eye_off from "@/assets/icons/eye-off.svg?raw";
    import icon_folder from "@/assets/icons/folder.svg?raw";
    import icon_arrow_down from "@/assets/icons/arrow-down.svg?raw";
    import icon_arrow_right from "@/assets/icons/arrow-right.svg?raw";

    let pane: HTMLDivElement | undefined = $state();
    let list: HTMLDivElement | undefined = $state();

</script>

<style>

    .rooms {
        position: relative;
        display: flex;
        flex-direction: column;
        flex: none;
        border-right: 1px solid var(--border);
        background: var(--component);
        overflow: hidden;
    }

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

    .list {
        flex: 1;
        display: flex;
        flex-direction: column;
    }

    /* Empty space below the last row, the end of the top level */
    .end {
        position: relative;
        flex: 1;
    }

    .list.moving {
        cursor: grabbing;
    }

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

    button {
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

    .selected {
        background-color: var(--component-hover);
    }

    .nested {
        padding-left: 2rem;
    }

    .held {
        opacity: 0.5;
    }

    /* Drawn over the row's edge so nothing moves */
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

    .drop-into {
        outline: 2px solid var(--btn-action);
        outline-offset: -2px;
    }

</style>

<!-- Layout for movable room button -->
{#snippet roomButton(roomId: string, nested: boolean)}
    {@const room = getRoomData(roomId)}
    <button
        class:nested
        class:selected={getSelectedRoom() === roomId}
        class:held={isHeld(roomId)}
        class:drop-above={dropAt(roomId) === Drop.Above}
        class:drop-below={dropAt(roomId) === Drop.Below}
        onpointerdown={(event) => startDrag(event, roomId)}
        onpointermove={(event) => hover(event, roomId)}
        onclick={() => { if (isLocked()) selectRoom(roomId); }}
        oncontextmenu={(event) => openMenu(event, roomCtx(roomId))}
    >
        {room?.name}
        {#if room?.visibility === "locked"}
            <span class="icon" aria-hidden="true" title="Locked">{@html icon_lock}</span>
        {:else if room?.visibility === "hidden"}
            <span class="icon" aria-hidden="true" title="Hidden">{@html icon_eye_off}</span>
        {/if}
    </button>
{/snippet}

<!-- Layout for movable folder -->
{#snippet folderButton(folder: Folder)}
    {#if isRenamingFolder(folder)}
        <input
            type="text"
            autofocus
            value={folder.name}
            onkeydown={(event) => finishRenamingFolder(event, folder)}
            onblur={stopRenamingFolder}
        />
    {:else}
        <button
            class:held={isHeld(folder)}
            class:drop-above={dropAt(folder) === Drop.Above}
            class:drop-below={dropAt(folder) === Drop.Below}
            class:drop-into={dropAt(folder) === Drop.Inside}
            onpointerdown={(event) => startDrag(event, folder)}
            onpointermove={(event) => hover(event, folder)}
            onclick={() => toggleFolder(folder)}
            oncontextmenu={(event) => openMenu(event, folderCtx(folder))}
        >
            <span class="icon" aria-hidden="true" title="Folder">{@html icon_folder}</span>
            {folder.name}
            {#if folder.collapsed}
                <span class="icon" aria-hidden="true" title="Expand Folder">{@html icon_arrow_down}</span>
            {:else}
                <span class="icon" aria-hidden="true" title="Collapse Folder">{@html icon_arrow_right}</span>
            {/if}
        </button>
    {/if}
{/snippet}


<!-- Release can happen anywhere, not only over the list -->
<svelte:window onpointerup={endDrag} onpointercancel={cancelDrag}/>

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
        {#if !isLocked()}
            <span class="icon" aria-hidden="true">{@html icon_lock_open}</span>
        {/if}
    </div>

    <!-- Room List -->
    <div
        class="list"
        class:moving={isDragging()}
        role="presentation"
        bind:this={list}
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
        {#if isCreatingFolder()}
            <input
                type="text"
                autofocus
                onkeydown={finishCreatingFolder}
                onblur={stopCreatingFolder}
            />
        {/if}

        <!-- Empty space below the last row -->
        <div
            class="end"
            role="presentation"
            onpointermove={hoverEnd}
        ></div>
    </div>

    <!-- Vertical Bar for resizing room pane-->
    <div
        class="resize-handle"
        class:dragging={isResizing()}
        role="separator"
        aria-orientation="vertical"
        aria-label="Resize room list"
        onpointerdown={startResize}
        onpointermove={(event) => resize(event, pane)}
        onpointerup={endResize}
        onlostpointercapture={endResize}
        ondblclick={resetWidth}
    ></div>
</div>
