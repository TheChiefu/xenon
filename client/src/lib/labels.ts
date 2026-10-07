// Text the client shows for each value

import { GlobalRole, Permission, Platform, Status, Visibility } from "@/bindings/shared";
import { Device } from "@/bindings/sockets/events";
import { AnimatePhotos } from "@/lib/settings.svelte";
import icon_steam from "@/assets/platforms/steam.svg?raw";
import icon_xbox from "@/assets/platforms/xbox.svg?raw";
import icon_android from "@/assets/devices/android.svg?raw";
import icon_chrome from "@/assets/devices/chrome.svg?raw";
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
  [Device.chrome]: icon_chrome,
  [Device.desktop]: icon_desktop,
  [Device.mobile]: icon_mobile,
  [Device.tablet]: icon_tablet,
};

export const ICONS_PLATFORM: Record<Platform, string> = {
  [Platform.xbox]: icon_xbox,
  [Platform.steam]: icon_steam,
};

export const LABELS_ANIMATE_PHOTOS: Record<AnimatePhotos, string> = {
  [AnimatePhotos.Always]: "Always",
  [AnimatePhotos.OnHover]: "On Hover",
  [AnimatePhotos.Never]: "Never",
};

export const LABELS_DEVICE: Record<Device, string> = {
  [Device.windows]: "Windows",
  [Device.macos]: "Mac",
  [Device.linux]: "Linux",
  [Device.android]: "Android",
  [Device.ios]: "iOS",
  [Device.chrome]: "Chrome",
  [Device.desktop]: "Desktop",
  [Device.mobile]: "Phone",
  [Device.tablet]: "Tablet",
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
  [Status.offline]: "Offline",
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
