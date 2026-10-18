import { LocalEvent } from "@/lib/events";
import * as events from "@/lib/events";

// Image shown in the viewer
export interface ViewedImage {
  url: string;
  filename: string;
}

// State //
let visible: boolean = $state(false);          // Dialog is on screen
let image: ViewedImage | null = $state(null);  // Image that is shown

// Getters //

export function isOpen(): boolean {
  return visible;
}

export function getImage(): ViewedImage | null {
  return image;
}

// Dialog //

export function close(): void {
  visible = false;
}

export function open(selected: ViewedImage): void {
  image = selected;
  visible = true;
}

// Open a selected image
events.subscribe(LocalEvent.image_selected, open);
