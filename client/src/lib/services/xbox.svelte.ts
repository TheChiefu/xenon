import type { FriendResponse } from "@/lib/api/xbox";
import { Platform } from "@/bindings/types/platforms";
import * as profile from "@/lib/users/profile.svelte";

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
