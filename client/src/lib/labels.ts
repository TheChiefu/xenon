// Text the client shows for each value

import { GlobalRole } from "@/bindings/types/users";
import { Notify, Permission, Visibility } from "@/bindings/types/rooms";
import { Availability, Platform } from "@/bindings/types/platforms";
import { Appearance } from "@/bindings/types/appearance";
import { CallCloseCode } from "@/bindings/sockets/call/events";
import { Device } from "@/bindings/types/appearance";
import { AnimatePhotos, isLightScheme, SendKey } from "@/lib/client/settings.svelte";
import icon_steam from "@/assets/platforms/steam.svg?raw";
import icon_steam_outline from "@/assets/platforms/steam_outline.svg?raw";
import icon_xbox from "@/assets/platforms/xbox.svg?raw";
import icon_android from "@/assets/devices/android.svg?raw";
import icon_desktop from "@/assets/devices/desktop.svg?raw";
import icon_ios from "@/assets/devices/ios.svg?raw";
import icon_linux from "@/assets/devices/linux.svg?raw";
import icon_mac from "@/assets/devices/mac.svg?raw";
import icon_mobile from "@/assets/devices/mobile.svg?raw";
import icon_tablet from "@/assets/devices/tablet.svg?raw";
import icon_windows from "@/assets/devices/windows.svg?raw";

export const ICONS_DEVICE: Record<Device, string> = {
  [Device.windows]: icon_windows,
  [Device.macos]: icon_mac,
  [Device.linux]: icon_linux,
  [Device.android]: icon_android,
  [Device.ios]: icon_ios,
  [Device.desktop]: icon_desktop,
  [Device.mobile]: icon_mobile,
  [Device.tablet]: icon_tablet,
};

export const ICONS_PLATFORM: Record<Platform, string> = {
  [Platform.xbox]: icon_xbox,
  get [Platform.steam]() {
    return isLightScheme() ? icon_steam_outline : icon_steam;
  },
};

export const LABELS_ANIMATE_PHOTOS: Record<AnimatePhotos, string> = {
  [AnimatePhotos.Always]: "Always",
  [AnimatePhotos.OnHover]: "On Hover",
  [AnimatePhotos.Never]: "Never",
};

export const LABELS_CALL_CLOSE: Record<CallCloseCode, string> = {
  [CallCloseCode.server_error]: "The server could not add you to the call",
  [CallCloseCode.no_connect]: "You do not have permission to join calls in this room",
  [CallCloseCode.already_in_call]: "You are already in a call",
  [CallCloseCode.call_full]: "The call is full",
};

export const LABELS_DEVICE: Record<Device, string> = {
  [Device.windows]: "Windows",
  [Device.macos]: "Mac",
  [Device.linux]: "Linux",
  [Device.android]: "Android",
  [Device.ios]: "iOS",
  [Device.desktop]: "Desktop",
  [Device.mobile]: "Phone",
  [Device.tablet]: "Tablet",
};

export const LABELS_NOTIFY: Record<Notify, string> = {
  [Notify.all]: "Every message",
  [Notify.mentions]: "Mentions only",
  [Notify.none]: "Nothing",
};

export const LABELS_PERMISSION: Record<Permission, string> = {
  [Permission.post]: "Send messages",
  [Permission.attach]: "Attach files",
  [Permission.delete_msg]: "Delete others' messages",
  [Permission.invite]: "Create invites",
  [Permission.rename]: "Edit name and visibility",
  [Permission.ban]: "Remove members",
  [Permission.grant]: "Set members' permissions",
  [Permission.delete_room]: "Delete the room",
  [Permission.connect]: "Join voice",
  [Permission.speak]: "Speak in voice",
  [Permission.mute]: "Mute others",
  [Permission.video]: "Show video / screenshare",
};

export const LABELS_PLATFORM: Record<Platform, string> = {
  [Platform.xbox]: "Xbox",
  [Platform.steam]: "Steam",
};

export const LABELS_AVAILABILITY: Record<Availability, string> = {
  [Availability.online]: "Online",
  [Availability.away]: "Away",
  [Availability.offline]: "Offline",
};

export const LABELS_ROLE: Record<GlobalRole, string> = {
  [GlobalRole.admin]: "Admin",
  [GlobalRole.member]: "Member",
  [GlobalRole.visitor]: "Visitor",
};

export const LABELS_APPEARANCE: Record<Appearance, string> = {
  [Appearance.online]: "Online",
  [Appearance.busy]: "Do Not Disturb",
  [Appearance.away]: "Away",
  [Appearance.offline]: "Offline",
};

export const LABELS_VISIBILITY: Record<Visibility, string> = {
  [Visibility.public]: "Public",
  [Visibility.hidden]: "Hidden",
};

export const LABELS_SEND_KEY: Record<SendKey, string> = {
  [SendKey.Enter]: "Enter",
  [SendKey.CtrlEnter]: "Ctrl+Enter",
  [SendKey.ShiftEnter]: "Shift+Enter",
};

export const CHOICE_YES = "Yes";
export const CHOICE_NO = "No";
export const CHOICE_DELETE = "Delete";
export const CHOICE_DELETE_WITH_MESSAGES = "Delete with Messages";
export const CHOICE_CANCEL = "Cancel";

export const MSG_SAVED = "Changes Saved";
export const MSG_REFRESHED = "Refreshed";
export const MSG_ERROR = "Error";
export const MSG_REMOVE_PUSH = "Are you sure you want to disable push notifications?";
export const MSG_DISABLE_NOTIFICATION = "Browser notifications must be revoked via browser settings.";
export const MSG_DELETE_ACCOUNT = "Are you sure you want to delete your account? This cannot be undone.";
export const MSG_UNLINK_XBOX = "Are you sure you want to unlink your Xbox account?";
export const MSG_UNLINK_STEAM = "Are you sure you want to unlink your Steam account?";
export const MSG_OPEN_SIGN_IN = "Continue to sign-in";
export const MSG_SIGN_IN_READY = "Sign-in link ready";
export const MSG_REFRESH_AFTER_SIGN_IN = "Refresh once signed in";
export const MSG_REVOKE_CODE = "Are you sure you want to revoke this invite code?";
export const MSG_DELETE_MESSAGE = "Are you sure you want to delete this message? This cannot be undone.";
export const MSG_DELETE_USER = "Are you sure you want to delete this user? This cannot be undone.";
export const MSG_LEAVE_ROOM = "Are you sure you want to leave this room?";
export const MSG_DELETE_ROOM = "Are you sure you want to delete this room? This cannot be undone.";
export const MSG_BAN_USER = "Are you sure you want to ban this user? They will be removed from the room.";
export const MSG_SERVER_BAN = "Are you sure you want to ban this user from the server? They will be signed out everywhere.";
export const MSG_MICROPHONE_DENIED = "Microphone access was denied";
export const MSG_LOADING = "Loading…";
export const MSG_LOAD_FAILED = "Failed to load";
export const MSG_PROMOTE_ADMIN = "Are you sure you want to make this user an admin? Admins cannot change each other's roles, so this cannot be undone.";

// Returns message for a successful refresh, stamped with the current time
export function refreshedText(): string {
  const now = new Date();
  return `(${MSG_REFRESHED} ${now.toLocaleTimeString('en-US', { hour12: true })})`;
}

export const DESCRIPTIONS_VISIBILITY: Record<Visibility, string> = {
  [Visibility.public]: "Listed in discovery, anyone can join",
  [Visibility.hidden]: "Not listed, joining needs an invite",
};
