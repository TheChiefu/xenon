<script lang="ts">
    import { getLayout, getRoomData, getSelectedRoom, selectRoom } from "@/lib/rooms.svelte";
    import { cancelRoom, dragRoom, dropRoom, getRoomTransform, holdRoom, isHeld, isMoving } from "@/lib/room_drag.svelte";
    import { endResize, getWidth, isResizing, resetWidth, resize, startResize } from "@/lib/room_resize.svelte";

    // Resize handle
    const HANDLE_WIDTH: number = 8;

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

    .held {
        position: relative;
        z-index: 1;
        background-color: var(--btn-action);
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
        white-space: nowrap;
        overflow: hidden;
        text-overflow: ellipsis;
        touch-action: none;
    }

    .selected {
        background-color: var(--component-hover);
    }

</style>

<div class="rooms" style="width: {getWidth()}px" style:--handle-width="{HANDLE_WIDTH}px" bind:this={pane}>

    <div class="header">
        <h3>Rooms</h3>
    </div>
    <hr/>
    <div
        class="list"
        class:moving={isMoving()}
        role="presentation"
        bind:this={list}
        onpointermove={(event) => dragRoom(event, list)}
        onpointerup={dropRoom}
        onpointercancel={cancelRoom}
    >
        {#each getLayout() as entry, index (entry.id)}
        <button
            class:selected={getSelectedRoom() === entry.id}
            class:held={isHeld(index)}
            style:transform={getRoomTransform(index)}
            onpointerdown={(event) => holdRoom(event, index)}
            onclick={() => selectRoom(entry.id)}
        >
            {getRoomData(entry.id)?.name}
        </button>
        {:else}
            <div>No rooms</div>
        {/each}
    </div>

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
