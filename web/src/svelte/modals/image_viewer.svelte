<script lang="ts">
  // Modal window showing the image stored by the viewer, zoomed by scroll, pinch or buttons and moved by dragging

  import * as image_viewer from "@/lib/display/image_viewer.svelte";
  import { dialogFly } from "@/lib/display/transitions";
  import icon_fit from "@/assets/icons/fit.svg?raw";
  import icon_x from "@/assets/icons/x.svg?raw";
  import icon_zoom_in from "@/assets/icons/zoom-in.svg?raw";
  import icon_zoom_out from "@/assets/icons/zoom-out.svg?raw";

  interface Point {
    x: number;
    y: number;
  }

  interface Props {
    onClose: () => void;
  }
  let { onClose }: Props = $props();
  let dialog = $state<HTMLDialogElement | null>(null);
  let stage = $state<HTMLDivElement | null>(null);

  const MIN_SCALE: number = 0.5;
  const MAX_SCALE: number = 8;
  const BUTTON_FACTOR: number = 1.25;
  const WHEEL_RATE: number = 0.0015;

  const image = $derived(image_viewer.getImage());

  // Zoom level, and offset of the image from the stage center in pixels
  let scale: number = $state(1);
  let offset_x: number = $state(0);
  let offset_y: number = $state(0);

  // Last position of each pointer pressed on the stage, by pointer ID
  const pointers: Map<number, Point> = new Map();

  $effect(() => {
    dialog?.showModal();
  });

  function cancel(event: Event) {
    event.preventDefault();
    onClose();
  }

  function pressAway(event: PointerEvent) {
    if (event.button === 0 && (event.target === dialog || event.target === stage)) {
      onClose();
    }
  }

  // Zooms by a factor, keeping a point on the stage in place
  // - factor: Amount the zoom level is multiplied by
  // - point: Position in the window that stays in place
  function zoom(factor: number, point: Point): void {
    if (stage === null) {
      return;
    }

    const next = Math.min(MAX_SCALE, Math.max(MIN_SCALE, scale * factor));

    // Get the point relative to the stage center
    const rect = stage.getBoundingClientRect();
    const center_x = point.x - (rect.left + rect.width / 2);
    const center_y = point.y - (rect.top + rect.height / 2);

    // Move the image so the point stays in place at the new zoom level
    offset_x = center_x - (center_x - offset_x) * (next / scale);
    offset_y = center_y - (center_y - offset_y) * (next / scale);
    scale = next;
  }

  // Zooms by a factor around the stage center
  // - factor: Amount the zoom level is multiplied by
  function zoomCenter(factor: number): void {
    if (stage === null) {
      return;
    }

    const rect = stage.getBoundingClientRect();
    zoom(factor, { x: rect.left + rect.width / 2, y: rect.top + rect.height / 2 });
  }

  // Returns the image to its fitted size and position
  function fit(): void {
    scale = 1;
    offset_x = 0;
    offset_y = 0;
  }

  function onWheel(event: WheelEvent): void {
    event.preventDefault();

    // Scroll up zooms in, scroll down zooms out
    const factor = Math.exp(-event.deltaY * WHEEL_RATE);
    zoom(factor, { x: event.clientX, y: event.clientY });
  }

  function onPointerDown(event: PointerEvent): void {

    // Only a left click, middle click or touch moves the image
    if (event.button !== 0 && event.button !== 1) {
      return;
    }

    // Keep receiving moves for the pointer when it leaves the stage
    stage?.setPointerCapture(event.pointerId);
    pointers.set(event.pointerId, { x: event.clientX, y: event.clientY });
  }

  function onPointerMove(event: PointerEvent): void {
    const previous = pointers.get(event.pointerId);
    if (previous === undefined) {
      return;
    }

    const current: Point = { x: event.clientX, y: event.clientY };

    // Drag the image with one pointer
    if (pointers.size === 1) {
      offset_x += current.x - previous.x;
      offset_y += current.y - previous.y;
    }

    // Pinch zoom with two pointers, around their midpoint
    if (pointers.size === 2) {
      const other = otherPointer(event.pointerId);

      if (other !== null) {
        const before = Math.hypot(previous.x - other.x, previous.y - other.y);
        const after = Math.hypot(current.x - other.x, current.y - other.y);
        const midpoint: Point = { x: (current.x + other.x) / 2, y: (current.y + other.y) / 2 };

        if (before > 0) {
          zoom(after / before, midpoint);
        }
      }
    }

    pointers.set(event.pointerId, current);
  }

  function onPointerUp(event: PointerEvent): void {
    pointers.delete(event.pointerId);
  }

  // Gets the position of the pressed pointer that is not the given one
  // - id: Pointer ID to leave out
  function otherPointer(id: number): Point | null {
    for (const [pointer_id, point] of pointers) {
      if (pointer_id !== id) {
        return point;
      }
    }

    return null;
  }
</script>

<style>
  dialog {
    width: 100%;
    height: 100%;
    max-width: none;
    max-height: none;
    margin: 0;
    padding: 0;
    border: none;
    background: transparent;
  }

  dialog::backdrop {
    background: var(--viewer-background);
  }

  .stage {
    width: 100%;
    height: 100%;
    overflow: hidden;
    touch-action: none;
    display: flex;
    align-items: center;
    justify-content: center;
  }

  .stage img {
    max-width: 100%;
    max-height: 100%;
    user-select: none;
    cursor: grab;
  }

  .controls {
    position: absolute;
    top: var(--section-padding);
    right: var(--section-padding);
    display: flex;
    gap: var(--inline-gap);
    border: 1px solid var(--border);
    background: var(--component);
  }
</style>

<dialog bind:this={dialog} onclose={onClose} oncancel={cancel} onpointerdown={pressAway} transition:dialogFly={{ y: "1rem" }}>
  {#if image !== null}
    <div
      class="stage"
      role="application"
      aria-label={image.filename}
      bind:this={stage}
      onwheel={onWheel}
      onpointerdown={onPointerDown}
      onpointermove={onPointerMove}
      onpointerup={onPointerUp}
      onpointercancel={onPointerUp}
    >
      <img
        src={image.url}
        alt={image.filename}
        draggable="false"
        style:transform={`translate(${offset_x}px, ${offset_y}px) scale(${scale})`}
      />
    </div>
  {/if}

  <div class="controls">
    <button class="icon-btn" aria-label="Zoom in" title="Zoom In" onclick={() => zoomCenter(BUTTON_FACTOR)}>
      <span class="icon" aria-hidden="true">{@html icon_zoom_in}</span>
    </button>
    <button class="icon-btn" aria-label="Zoom out" title="Zoom Out" onclick={() => zoomCenter(1 / BUTTON_FACTOR)}>
      <span class="icon" aria-hidden="true">{@html icon_zoom_out}</span>
    </button>
    <button class="icon-btn" aria-label="Fit" title="Fit" onclick={fit}>
      <span class="icon" aria-hidden="true">{@html icon_fit}</span>
    </button>
    <button class="icon-btn" aria-label="Close" title="Close" onclick={onClose}>
      <span class="icon" aria-hidden="true">{@html icon_x}</span>
    </button>
  </div>
</dialog>
