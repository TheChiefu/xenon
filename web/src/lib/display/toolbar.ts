
// Pane toggle shown in a toolbar
export interface ToolbarButton {
  label: string;        // Accessible name and tooltip
  icon: string;         // Raw SVG markup
  active: boolean;      // Whether the pane is open
  kind: Kind; // Type of button
  badge?: number;
  onclick: () => void;
}

export enum Kind {
  Regular,
  Overlay,
  Modal,
}