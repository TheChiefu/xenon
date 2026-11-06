// Sounds played when someone joins or leaves the call

import join_url from "@/assets/sounds/join.ogg?url";
import leave_url from "@/assets/sounds/leave.ogg?url";

export function playJoin(): void {
  play(join_url);
}

export function playLeave(): void {
  play(leave_url);
}

// HELPERS //

function play(url: string): void {
  new Audio(url).play().catch(console.warn);
}
