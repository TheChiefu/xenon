// State of the Yes/No popup mounted at the app root (see app.svelte)
// Shown by publishing LocalEvent.confirm_requested, one question at a time

import { LocalEvent, type ConfirmRequest } from "@/lib/events";
import * as events from "@/lib/events";

let message = $state("");
let visible = $state(false);
let current: ConfirmRequest | null = null;
const queue: ConfirmRequest[] = [];

export function isVisible(): boolean {
  return visible;
}

export function getMessage(): string {
  return message;
}

// Called by the root Popup instance's onConfirm/onCancel
export function resolve(result: boolean): void {
  current?.answer(result);
  showNext();
}

// Shows the oldest queued question, or hides the popup when none are queued
function showNext(): void {
  const next = queue.shift();

  if (next === undefined) {
    current = null;
    visible = false;
    return;
  }

  current = next;
  message = next.message;
  visible = true;
}

// Queue a question, shown once the questions before it are answered
events.subscribe(LocalEvent.confirm_requested, (request: ConfirmRequest) => {
  queue.push(request);

  if (!visible) {
    showNext();
  }
});

// Answer No to the shown question and every queued question on sign out
events.subscribe(LocalEvent.signed_out, () => {
  for (const request of queue.splice(0)) {
    request.answer(false);
  }

  if (visible) {
    resolve(false);
  }
});
