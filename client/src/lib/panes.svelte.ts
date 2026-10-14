const BREAKPOINT: number = 1000;
const MEMBERS_WIDTH: number = 16;
const ROOM_WIDTH: number = 16;
const XBOX_WIDTH: number = 20;

export enum Pane {
  room = "room",
  members = "members",
  xbox = "xbox",
}

interface Definition {
  width: number;
  open: boolean;
}

const PANES: Record<Pane, Definition> = {
  [Pane.room]: {
    width: ROOM_WIDTH,
    open: true,
  },
  [Pane.members]: {
    width: MEMBERS_WIDTH,
    open: true,
  },
  [Pane.xbox]: {
    width: XBOX_WIDTH,
    open: false,
  },
};

const open: Record<Pane, boolean> = $state({
  [Pane.room]: initialOpen(Pane.room),
  [Pane.members]: initialOpen(Pane.members),
  [Pane.xbox]: initialOpen(Pane.xbox),
});

function openKey(pane: Pane): string {
  return `pane_open_${pane}`;
}

function initialOpen(pane: Pane): boolean {
  const stored: string | null = localStorage.getItem(openKey(pane));

  return stored !== null ? stored === "true" : PANES[pane].open;
}

export function getWidth(pane: Pane): number {
  return PANES[pane].width;
}

export function isOpen(pane: Pane): boolean {
  return open[pane];
}

export function togglePane(pane: Pane): void {
  open[pane] = !open[pane];

  if (window.innerWidth >= BREAKPOINT) {
    localStorage.setItem(openKey(pane), String(open[pane]));
    return;
  }

  for (const other of Object.values(Pane)) {
    if (other === pane) continue;

    open[other] = false;
  }
}
