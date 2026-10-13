import type { FriendResponse } from "@/bindings/services/xbox";
import { Platform } from "@/bindings/shared";
import { getXboxFriends } from "@/lib/api/xbox";
import { getProfile } from "@/lib/profile.svelte";
import { Edge, PaneWidth } from "@/lib/resize.svelte";

const KEY_USER_XBOX_WIDTH: string = "xbox_panel_width";

export const xboxWidth = new PaneWidth(KEY_USER_XBOX_WIDTH, Edge.left);  // Pane width

// State //
let visible: boolean = $state(false);        // Pane is on screen
let friends: FriendResponse[] = $state([]);  // Friends from last load

// Getters //

export function isXboxOpen(): boolean {
  return visible;
}

export function getFriends(): FriendResponse[] {
  return friends;
}

// Whether profile has a linked Xbox account
export function isXboxLinked(): boolean {
  const profile = getProfile();

  if (profile === null) {
    return false;
  }

  return profile.linked_accounts.some((account) => account.platform === Platform.xbox);
}

// Pane //

export function toggleXbox(): void {
  visible = !visible;
}

// Server //

export async function loadFriends(): Promise<void> {
  friends = await getXboxFriends();
}
