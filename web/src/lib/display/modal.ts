// Modal dialogs

// Attachment: opens the dialog as a modal, calling onClose on Escape or a press outside it
export function modal(onClose: () => void): (dialog: HTMLDialogElement) => () => void {
  return (dialog: HTMLDialogElement) => {

    // Escape calls onClose instead of closing the dialog directly
    const cancel = (event: Event) => {
      if (event.target === dialog) {
        event.preventDefault();
        onClose();
      }
    };

    // A press on the dialog element itself is a press on the backdrop
    const pressAway = (event: PointerEvent) => {
      if (event.target === dialog) {
        onClose();
      }
    };

    dialog.addEventListener("cancel", cancel);
    dialog.addEventListener("pointerdown", pressAway);
    dialog.showModal();

    return () => {
      dialog.removeEventListener("cancel", cancel);
      dialog.removeEventListener("pointerdown", pressAway);
    };
  };
}
