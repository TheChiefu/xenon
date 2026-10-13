// Pane width, changed by dragging a handle on one edge
export const HANDLE_WIDTH: number = 8;
const DEFAULT_WIDTH: number = 300;

// Edge of a pane holding its resize handle
export enum Edge {
  left,
  right,
}

export class PaneWidth {
  private key: string;  // Local storage key for saved width
  private edge: Edge;
  private width: number = $state(0);
  private resizing: boolean = $state(false);

  constructor(key: string, edge: Edge) {
    const stored: string | null = localStorage.getItem(key);

    this.key = key;
    this.edge = edge;
    this.width = stored !== null ? Number(stored) : DEFAULT_WIDTH;
  }

  // Getters and Setters //

  getWidth(): number {
    return this.width;
  }

  isResizing(): boolean {
    return this.resizing;
  }

  reset(): void {
    this.width = DEFAULT_WIDTH;
    localStorage.setItem(this.key, String(this.width));
  }

  // Pointer Events //

  resizeStart(event: PointerEvent): void {
    const handle = event.currentTarget as HTMLElement;
    handle.setPointerCapture(event.pointerId);
    this.resizing = true;
  }

  resize(event: PointerEvent, pane: HTMLElement | undefined): void {
    if (!this.resizing || pane === undefined) return;

    // Measures from edge opposite resize handle
    const bounds: DOMRect = pane.getBoundingClientRect();
    const next: number = this.edge === Edge.right
      ? event.clientX - bounds.left
      : bounds.right - event.clientX;

    this.width = Math.min(Math.max(next, HANDLE_WIDTH), window.innerWidth);
  }

  resizeStop(): void {
    if (!this.resizing) return;

    this.resizing = false;
    localStorage.setItem(this.key, String(this.width));
  }
}
