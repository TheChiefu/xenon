<script lang="ts">
    import type { MyRoomResponse } from "@/bindings/routes/rooms";
    import { getLayout, isLocked } from "@/lib/rooms/layout.svelte";
    import { getRoomData } from "@/lib/rooms.svelte";
    import { getSelectedRoom, selectRoom } from "@/lib/rooms/selection.svelte";
    import { cancelDrag, currentIndex, endDrag, getTransform, isDragging, startDrag, trackDrag } from "@/lib/rooms/drag.svelte";
    import { endResize, getWidth, HANDLE_WIDTH, isResizing, resetWidth, resize, startResize } from "@/lib/rooms/resize.svelte";
    import { closeMenu, getMenuPos, onAddFolder, onNewRoom, onToggleLock, openMenu } from "@/lib/rooms/menu.svelte";
    import { cancelFolder, confirmFolder, isCollapsed, isCreatingFolder, toggleFolder } from "@/lib/rooms/folders.svelte";
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
        display: flex;
        flex-direction: column;
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

</style>

<!-- Layout for movable room button -->
{#snippet roomButton(
    roomId: string,
    nested: boolean,
    transform: string | null,
    onpointerdown: ((event: PointerEvent) => void) | undefined
)}
    {@const room = getRoomData(roomId)}
    <button
        class:nested={nested}
        class:selected={getSelectedRoom() === roomId}
        style:transform={transform}
        onpointerdown={onpointerdown}
        onclick={() => selectRoom(roomId)}
    >
        {room?.name}
        {#if room?.visibility === "locked"}
            <span class="icon" aria-hidden="true">{@html icon_lock}</span>
        {:else if room?.visibility === "hidden"}
            <span class="icon" aria-hidden="true">{@html icon_eye_off}</span>
        {/if}
    </button>
{/snippet}

<!-- Layout for movable folder -->
{#snippet folderButton(folderName: string, index: number)}
    <button
        style:transform={getTransform(index)}
        onpointerdown={(event) => startDrag(event, index)}
        onclick={() => toggleFolder(folderName)}
    >
        <span class="icon" aria-hidden="true">{@html icon_folder}</span>
        {folderName}
        {#if isCollapsed(folderName)}
            <span class="icon" aria-hidden="true">{@html icon_arrow_down}</span>
        {:else}
            <span class="icon" aria-hidden="true">{@html icon_arrow_right}</span>
        {/if}
    </button>
{/snippet}


<!-- Room Pane -->
<div
    class="rooms"
    style="width: {getWidth()}px"
    style:--handle-width="{HANDLE_WIDTH}px"
    oncontextmenu={openMenu}
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
        onpointermove={trackDrag}
        onpointerup={endDrag}
        onpointercancel={cancelDrag}
    >
        {#each getLayout() as entry, index}

            <!-- Top Level Room -->
            {#if typeof entry === "string"}
                {@render roomButton(
                    entry,
                    false,
                    getTransform(index),
                    (event) => startDrag(event, index)
                )}
            {:else}

            <!-- Folder -->
            {@render folderButton(entry.name, index)}
                <!-- Room inside folder -->
                {#if !isCollapsed(entry.name)}
                    {#each entry.rooms as roomId}
                    {@render roomButton(roomId, true, null, undefined)}
                    {/each}
                {/if}
            {/if}
        {/each}
    
        <!-- New Folder Input Bar -->
        {#if isCreatingFolder()}
        <input
            type="text"
            autofocus
            onkeydown={(event) => {
                if (event.key === "Enter") confirmFolder(event.currentTarget.value);
                if (event.key === "Escape") cancelFolder();
            }}
            onblur={(event) => confirmFolder(event.currentTarget.value)}
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
        onpointerdown={startResize}
        onpointermove={(event) => resize(event, pane)}
        onpointerup={endResize}
        onlostpointercapture={endResize}
        ondblclick={resetWidth}
    ></div>
</div>

<!-- Right click context window -->
{#if getMenuPos()}
    <div class="menu-backdrop" role="presentation" onpointerdown={closeMenu}></div>
    <div class="menu" style:left="{getMenuPos()?.x}px" style:top="{getMenuPos()?.y}px">
        <button onclick={onToggleLock}>{isLocked() ? "Unlock Layout" : "Lock Layout"}</button>
        <button onclick={onAddFolder}>Add Folder</button>
        <button onclick={onNewRoom}>New Room</button>
    </div>
{/if}
