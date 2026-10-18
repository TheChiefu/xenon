import { isCompact } from "@/lib/utils";

export enum Pane {
  search = "search",
  room = "room",
  members = "members",
  xbox = "xbox",
}

interface Definition {
  open: boolean;
}

const PANES: Record<Pane, Definition> = {
  [Pane.search]: {
    open: false,
  },
  [Pane.room]: {
    open: true,
  },
  [Pane.members]: {
    open: true,
  },
  [Pane.xbox]: {
    open: false,
  },
};

const open: Record<Pane, boolean> = $state({
  [Pane.search]: initialOpen(Pane.search),
  [Pane.room]: initialOpen(Pane.room),
  [Pane.members]: initialOpen(Pane.members),
  [Pane.xbox]: initialOpen(Pane.xbox),
});

function openKey(pane: Pane): string {
  return `pane_open_${pane}`;
}

// Starting state of a pane
function initialOpen(pane: Pane): boolean {

  // If view is compact, no panes are opened
  if (isCompact.current) return false;

  // Load saved pane
  const stored: string | null = localStorage.getItem(openKey(pane));

  // If nothing is stored, open default panes
  if (stored === null) {
    return PANES[pane].open;
  }

  // If stored pane is loaded, return open/close state
  return stored === "true";
}

export function isOpen(pane: Pane): boolean {
  return open[pane];
}

// Sets whether a pane is open
export function setOpenState(pane: Pane, value: boolean): void {
  open[pane] = value;

  if (isCompact.current) return;

  localStorage.setItem(openKey(pane), String(value));
}

export function toggle(pane: Pane): void {

  // Open/Close targeted pane
  setOpenState(pane, !open[pane]);

  // Wide view allows multiple panes to be open at once
  if (!isCompact.current) return;

  // Compact view closes other panes (avoid pane overlap)
  for (const other of Object.values(Pane)) {
    if (other === pane) continue;
    setOpenState(other, false);
  }
}
