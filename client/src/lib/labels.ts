// Text the client shows for each value

import { Device, GlobalRole, Permission, Platform, Status, Visibility } from "@/bindings/types";
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
  [Device.Windows]: icon_windows,
  [Device.Macos]: icon_mac,
  [Device.Linux]: icon_linux,
  [Device.Android]: icon_android,
  [Device.Ios]: icon_ios,
  [Device.Chrome]: icon_chrome,
  [Device.Desktop]: icon_desktop,
  [Device.Mobile]: icon_mobile,
  [Device.Tablet]: icon_tablet,
};

export const ICONS_PLATFORM: Record<Platform, string> = {
  [Platform.Xbox]: icon_xbox,
  [Platform.Steam]: icon_steam,
};

export const LABELS_DEVICE: Record<Device, string> = {
  [Device.Windows]: "Windows",
  [Device.Macos]: "Mac",
  [Device.Linux]: "Linux",
  [Device.Android]: "Android",
  [Device.Ios]: "iOS",
  [Device.Chrome]: "Chrome",
  [Device.Desktop]: "Desktop",
  [Device.Mobile]: "Phone",
  [Device.Tablet]: "Tablet",
};

export const LABELS_PERMISSION: Record<Permission, string> = {
  [Permission.Post]: "Send messages",
  [Permission.Attach]: "Attach files",
  [Permission.Commands]: "Use commands",
  [Permission.DeleteMsg]: "Delete others' messages",
  [Permission.Invite]: "Create invites",
  [Permission.Rename]: "Edit name and visibility",
  [Permission.Ban]: "Remove members",
  [Permission.Grant]: "Set members' permissions",
  [Permission.DeleteRoom]: "Delete the room",
  [Permission.Connect]: "Join voice",
  [Permission.Speak]: "Speak in voice",
  [Permission.Mute]: "Mute others",
  [Permission.Video]: "Show video",
  [Permission.Screenshare]: "Share screen",
};

export const LABELS_PLATFORM: Record<Platform, string> = {
  [Platform.Xbox]: "Xbox",
  [Platform.Steam]: "Steam",
};

export const LABELS_ROLE: Record<GlobalRole, string> = {
  [GlobalRole.Owner]: "Owner",
  [GlobalRole.Admin]: "Admin",
  [GlobalRole.Member]: "Member",
  [GlobalRole.Visitor]: "Visitor",
};

export const LABELS_STATUS: Record<Status, string> = {
  [Status.Online]: "Online",
  [Status.Busy]: "Do Not Disturb",
  [Status.Away]: "Away",
  [Status.Offline]: "Offline",
};

export const LABELS_VISIBILITY: Record<Visibility, string> = {
  [Visibility.Public]: "Public",
  [Visibility.Locked]: "Locked",
  [Visibility.Hidden]: "Hidden",
};

export const DESCRIPTIONS_VISIBILITY: Record<Visibility, string> = {
  [Visibility.Public]: "Listed in discovery, anyone can join",
  [Visibility.Locked]: "Listed in discovery, joining needs an invite",
  [Visibility.Hidden]: "Not listed, joining needs an invite",
};
