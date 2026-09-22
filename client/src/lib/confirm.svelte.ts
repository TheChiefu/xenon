// Shared Yes/No confirmation popup - one instance is mounted at the app root
// (see app.svelte); call confirmDialog() from anywhere to show it.

let message = $state("");
let visible = $state(false);
let resolveFn: ((result: boolean) => void) | null = null;

export function isConfirmVisible(): boolean {
  return visible;
}

export function getConfirmMessage(): string {
  return message;
}

// Show the popup and wait for the user's answer
export function confirmDialog(text: string): Promise<boolean> {
  message = text;
  visible = true;

  return new Promise<boolean>((resolve) => {
    resolveFn = resolve;
  });
}

// Called by the root Popup instance's onConfirm/onCancel
export function resolveConfirm(result: boolean): void {
  visible = false;
  resolveFn?.(result);
  resolveFn = null;
}
