// Collapsing menu dialogs

import { isCompact } from "@/lib/utils";

export const COLLAPSED_CLASS: string = "is-collapsed";

// Attachment: adds is-collapsed while the dialog is narrower than its set width
export function collapsible(dialog: HTMLDialogElement): () => void {
  const observer = new ResizeObserver(() => {
    const drawn_width = dialog.getBoundingClientRect().width;

    // Measure the width with max-width removed
    dialog.style.maxWidth = "none";
    const set_width = dialog.getBoundingClientRect().width;
    dialog.style.maxWidth = "";

    // Collapsed in compact mode, or when drawn more than half a pixel under the set width
    const collapsed = isCompact.current || drawn_width < set_width - 0.5;
    dialog.classList.toggle(COLLAPSED_CLASS, collapsed);
  });

  observer.observe(dialog);
  return () => observer.disconnect();
}
