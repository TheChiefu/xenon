import type { Pane } from "@/lib/panes.svelte";

// Pane toggle shown in a toolbar
export interface ToolbarButton {
  pane: Pane;           // Pane the button toggles
  label: string;        // Accessible name and tooltip
  icon: string;         // Raw SVG markup
  active: boolean;      // Whether the pane is open
  onclick: () => void;
}
