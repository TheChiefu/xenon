import type { FriendResponse } from "@/bindings/services/xbox";
import { Platform } from "@/bindings/shared";
import { getXboxFriends } from "@/lib/api/xbox";
import { getProfile } from "@/lib/profile.svelte";

// State //
let friends: FriendResponse[] = $state([]);  // Friends from last load

// Getters //

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

// Server //

export async function loadFriends(): Promise<void> {
  friends = await getXboxFriends();
}
