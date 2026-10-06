// Text the client shows for each value

import { GlobalRole, Permission, Platform, Status, Visibility } from "@/bindings/shared";
import { AnimatePhotos } from "@/lib/settings.svelte";
import icon_steam from "@/assets/platforms/steam.svg?raw";
import icon_xbox from "@/assets/platforms/xbox.svg?raw";

export const ICONS_PLATFORM: Record<Platform, string> = {
  [Platform.xbox]: icon_xbox,
  [Platform.steam]: icon_steam,
};

export const LABELS_ANIMATE_PHOTOS: Record<AnimatePhotos, string> = {
  [AnimatePhotos.Always]: "Always",
  [AnimatePhotos.OnHover]: "On Hover",
  [AnimatePhotos.Never]: "Never",
};

export const LABELS_PERMISSION: Record<Permission, string> = {
  [Permission.post]: "Send messages",
  [Permission.attach]: "Attach files",
  [Permission.commands]: "Use commands",
  [Permission.delete_msg]: "Delete others' messages",
  [Permission.invite]: "Create invites",
  [Permission.rename]: "Edit name and visibility",
  [Permission.ban]: "Remove members",
  [Permission.grant]: "Set members' permissions",
  [Permission.delete_room]: "Delete the room",
  [Permission.connect]: "Join voice",
  [Permission.speak]: "Speak in voice",
  [Permission.mute]: "Mute others",
  [Permission.video]: "Show video",
  [Permission.screenshare]: "Share screen",
};

export const LABELS_PLATFORM: Record<Platform, string> = {
  [Platform.xbox]: "Xbox",
  [Platform.steam]: "Steam",
};

export const LABELS_ROLE: Record<GlobalRole, string> = {
  [GlobalRole.owner]: "Owner",
  [GlobalRole.admin]: "Admin",
  [GlobalRole.member]: "Member",
  [GlobalRole.visitor]: "Visitor",
};

export const LABELS_STATUS: Record<Status, string> = {
  [Status.online]: "Online",
  [Status.busy]: "Do Not Disturb",
  [Status.away]: "Away",
  [Status.invisible]: "Invisible",
};

export const LABELS_VISIBILITY: Record<Visibility, string> = {
  [Visibility.public]: "Public",
  [Visibility.locked]: "Locked",
  [Visibility.hidden]: "Hidden",
};

export const DESCRIPTIONS_VISIBILITY: Record<Visibility, string> = {
  [Visibility.public]: "Listed in discovery, anyone can join",
  [Visibility.locked]: "Listed in discovery, joining needs an invite",
  [Visibility.hidden]: "Not listed, joining needs an invite",
};
