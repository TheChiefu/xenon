<script lang="ts">
    import type { Folder } from "@/bindings/shared";
    import { Visibility } from "@/bindings/shared";
    import { LABELS_VISIBILITY } from "@/lib/labels";
    import { Section } from "@/lib/rooms/layout.svelte";
    import * as layout from "@/lib/rooms/layout.svelte";
    import type { RoomId } from "@/lib/rooms/data.svelte";
    import * as room from "@/lib/rooms/data.svelte";
    import * as session from "@/lib/session.svelte";
    import * as drag from "@/lib/rooms/drag.svelte";
    import * as menu from "@/lib/menu.svelte";
    import { folderCtx, paneCtx, roomCtx } from "@/lib/rooms/context_menu";
    import { nameFinished, isCreating, isRenaming, nameReset, startCreation } from "@/lib/rooms/folders.svelte";
    import * as creation from "@/lib/rooms/creation.svelte";
    import icon_plus from "@/assets/icons/plus.svg?raw";
    import icon_compass from "@/assets/icons/compass.svg?raw";
    import icon_copy from "@/assets/icons/copy.svg?raw";
    import icon_gear from "@/assets/icons/gear.svg?raw";
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
        flex-direction: column;
        flex: none;
    }

    .row {
        display: flex;
        align-items: center;
        gap: 0.1rem;
        padding: 0.5rem;
    }

    .action-row {
        padding: 0.25rem 0.5rem;
        border: 1px solid var(--border);
        background: var(--component-raised);
    }

    .room-actions {
        display: contents;
    }

    .header h3 {
        padding-right: var(--inline-gap);
        font-size: 1rem;
        line-height: 1;
    }

    .spacer {
        flex: 1;
    }

    .divider {
        align-self: stretch;
        width: 1px;
        background: var(--border);
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

    .list button > * {
        pointer-events: none;
    }

    .folder-name {
        flex: 1;
        margin-left: 0.5rem;
        overflow: hidden;
        text-overflow: ellipsis;
    }

    .selected {
        background-color: var(--btn-active);
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
        class:selected={session.getSelectedRoom() === roomId}
        class:held={drag.isHeld(roomId)}
        class:drop-above={drop === Section.Above}
        class:drop-below={drop === Section.Below}
        {@attach drag.dropTarget(roomId)}
        onpointerdown={(event) => drag.start(event, roomId)}
        onclick={() => { if (layout.isLocked()) session.selectRoom(roomId); }}
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
            {@attach drag.dropTarget(folder)}
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

<!-- Display Header and related buttons -->
<div class="header">

    <!-- Header Row -->
    <div class="row">
        <h3>Rooms</h3>
        <span class="divider" aria-hidden="true"></span>
        <span class="spacer"></span>
        <span class="divider" aria-hidden="true"></span>

        <!-- Discover Rooms -->
        <button
            class="icon-btn"
            aria-label="Discover rooms"
            title="Discover rooms"
        >
            <span class="icon" aria-hidden="true">{@html icon_compass}</span>
        </button>

        <!-- Layout Lock -->
        <button
            class="icon-btn"
            aria-label={layout.isLocked() ? "Unlock layout" : "Lock layout"}
            title={layout.isLocked() ? "Unlock layout" : "Lock layout"}
            onclick={layout.toggleLock}
        >
            <span class="icon" aria-hidden="true">{@html layout.isLocked() ? icon_lock : icon_lock_open}</span>
        </button>
    </div>

    <!-- Action Row -->
    <div class="row action-row">

        <!-- New Room -->
        <button
            class="icon-btn"
            aria-label="New room"
            title="New room"
            onclick={creation.open}
        >
            <span class="icon" aria-hidden="true">{@html icon_plus}</span>
        </button>

        <!-- New Folder, only while unlocked -->
        {#if !layout.isLocked()}
            <button
                class="icon-btn"
                aria-label="New folder"
                title="New folder"
                onclick={startCreation}
            >
                <span class="icon" aria-hidden="true">{@html icon_folder}</span>
            </button>
        {/if}

        <!-- Selected Room Actions, hidden without a room but still taking space -->
        <span class="spacer"></span>
        <div class="room-actions" style:visibility={session.getSelectedRoom() === null ? "hidden" : null}>
            <span class="divider" aria-hidden="true"></span>

            <!-- Close Room -->
            <button
                class="icon-btn"
                aria-label="Close room"
                title="Close room"
                onclick={() => session.selectRoom(null)}
            >
                <span class="icon" aria-hidden="true">{@html icon_back}</span>
            </button>

            <!-- Copy Room ID -->
            <button
                class="icon-btn"
                aria-label="Copy room ID"
                title="Copy room ID"
                onclick={() => navigator.clipboard.writeText(session.getSelectedRoom() ?? "")}
            >
                <span class="icon" aria-hidden="true">{@html icon_copy}</span>
            </button>

            <!-- Room Settings -->
            <button
                class="icon-btn"
                aria-label="Room settings"
                title="Room settings"
            >
                <span class="icon" aria-hidden="true">{@html icon_gear}</span>
            </button>
        </div>
    </div>
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
            onkeydown={nameFinished}
            onblur={nameReset}
        />
    {/if}
</div>
