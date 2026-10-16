import type { FriendResponse } from "@/bindings/services/xbox";
import { Platform } from "@/bindings/shared";
import * as xboxAPI from "@/lib/api/xbox";
import * as profile from "@/lib/profile.svelte";

// State //
let friends: FriendResponse[] = $state([]);  // Friends from last load

// Getters //

export function getFriends(): FriendResponse[] {
  return friends;
}

// Whether profile has a linked Xbox account
export function isLinked(): boolean {
  const my_profile = profile.get();

  if (my_profile === null) {
    return false;
  }

  return my_profile.linked_accounts.some((account) => account.platform === Platform.xbox);
}

// Server //

export async function loadFriends(): Promise<void> {
  friends = await xboxAPI.getFriends();
}
