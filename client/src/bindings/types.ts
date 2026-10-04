// Match Exactly the generated bindings in bindings folder

import type {
  GlobalRole as SharedGlobalRole,
  Notify as SharedNotify,
  Permission as SharedPermission,
  Platform as SharedPlatform,
  Status as SharedStatus,
  Visibility as SharedVisibility,
} from "./shared";

export const GlobalRole = {
  Owner: "owner",
  Admin: "admin",
  Member: "member",
  Visitor: "visitor",
} as const satisfies Record<string, SharedGlobalRole>;
export type GlobalRole = SharedGlobalRole;

export const Notify = {
  None: "none",
  Mentions: "mentions",
  All: "all",
} as const satisfies Record<string, SharedNotify>;
export type Notify = SharedNotify;

export const Permission = {
  Post: "post",
  Attach: "attach",
  Commands: "commands",
  DeleteMsg: "delete_msg",
  Invite: "invite",
  Rename: "rename",
  Ban: "ban",
  Grant: "grant",
  DeleteRoom: "delete_room",
  Connect: "connect",
  Speak: "speak",
  Mute: "mute",
  Video: "video",
  Screenshare: "screenshare",
} as const satisfies Record<string, SharedPermission>;
export type Permission = SharedPermission;

export const Platform = {
  Xbox: "xbox",
  Steam: "steam",
} as const satisfies Record<string, SharedPlatform>;
export type Platform = SharedPlatform;

export const Status = {
  Online: "online",
  Busy: "busy",
  Away: "away",
  Invisible: "invisible",
} as const satisfies Record<string, SharedStatus>;
export type Status = SharedStatus;

export const Visibility = {
  Public: "public",
  Locked: "locked",
  Hidden: "hidden",
} as const satisfies Record<string, SharedVisibility>;
export type Visibility = SharedVisibility;
