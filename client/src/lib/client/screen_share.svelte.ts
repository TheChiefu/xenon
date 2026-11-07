// Screen Share Settings - The resolution, framerate and bitrate a screen share is sent at,
// so a share fits the sharer's upload without lagging the viewers

import { LocalEvent } from "@/lib/events";
import * as events from "@/lib/events";

// WIDTH //
const keyWidth = "screen_share_width";
const defaultWidth = 1280;
export const MIN_WIDTH = 320;
export const MAX_WIDTH = 3840;

let currentWidth: number = $state(readWidth());

export function getWidth(): number {
  return currentWidth;
}

export function setWidth(width: number): void {
  if (!isValidWidth(width)) {
    return;
  }

  currentWidth = width;
  localStorage.setItem(keyWidth, String(width));
  events.dispatch(LocalEvent.screen_share_settings_changed);
}

function isValidWidth(width: number): boolean {
  if (!Number.isFinite(width)) {
    return false;
  }

  if (width < MIN_WIDTH) {
    return false;
  }

  if (width > MAX_WIDTH) {
    return false;
  }

  return true;
}

function readWidth(): number {
  const stored = Number(localStorage.getItem(keyWidth) ?? defaultWidth);

  // Fallback on invalid
  if (!isValidWidth(stored)) {
    console.log("Invalid screen share width, falling back to default");
    return defaultWidth;
  }

  return stored;
}

// HEIGHT //
const keyHeight = "screen_share_height";
const defaultHeight = 720;
export const MIN_HEIGHT = 240;
export const MAX_HEIGHT = 2160;

let currentHeight: number = $state(readHeight());

export function getHeight(): number {
  return currentHeight;
}

export function setHeight(height: number): void {
  if (!isValidHeight(height)) {
    return;
  }

  currentHeight = height;
  localStorage.setItem(keyHeight, String(height));
  events.dispatch(LocalEvent.screen_share_settings_changed);
}

function isValidHeight(height: number): boolean {
  if (!Number.isFinite(height)) {
    return false;
  }

  if (height < MIN_HEIGHT) {
    return false;
  }

  if (height > MAX_HEIGHT) {
    return false;
  }

  return true;
}

function readHeight(): number {
  const stored = Number(localStorage.getItem(keyHeight) ?? defaultHeight);

  // Fallback on invalid
  if (!isValidHeight(stored)) {
    console.log("Invalid screen share height, falling back to default");
    return defaultHeight;
  }

  return stored;
}

// FRAME RATE //
const keyFrameRate = "screen_share_frame_rate";
const defaultFrameRate = 30;
export const MIN_FRAME_RATE = 5;
export const MAX_FRAME_RATE = 240;

let currentFrameRate: number = $state(readFrameRate());

export function getFrameRate(): number {
  return currentFrameRate;
}

export function setFrameRate(frame_rate: number): void {
  if (!isValidFrameRate(frame_rate)) {
    return;
  }

  currentFrameRate = frame_rate;
  localStorage.setItem(keyFrameRate, String(frame_rate));
  events.dispatch(LocalEvent.screen_share_settings_changed);
}

function isValidFrameRate(frame_rate: number): boolean {
  if (!Number.isFinite(frame_rate)) {
    return false;
  }

  if (frame_rate < MIN_FRAME_RATE) {
    return false;
  }

  if (frame_rate > MAX_FRAME_RATE) {
    return false;
  }

  return true;
}

function readFrameRate(): number {
  const stored = Number(localStorage.getItem(keyFrameRate) ?? defaultFrameRate);

  // Fallback on invalid
  if (!isValidFrameRate(stored)) {
    console.log("Invalid screen share framerate, falling back to default");
    return defaultFrameRate;
  }

  return stored;
}

// BITRATE //
// The bitrate cap in kbps, 0 for no cap
const keyBitrate = "screen_share_bitrate";
const defaultBitrate = 2500;
export const MIN_BITRATE = 0;
export const MAX_BITRATE = 25000;

let currentBitrate: number = $state(readBitrate());

export function getBitrate(): number {
  return currentBitrate;
}

export function setBitrate(bitrate: number): void {
  if (!isValidBitrate(bitrate)) {
    return;
  }

  currentBitrate = bitrate;
  localStorage.setItem(keyBitrate, String(bitrate));
  events.dispatch(LocalEvent.screen_share_settings_changed);
}

function isValidBitrate(bitrate: number): boolean {
  if (!Number.isFinite(bitrate)) {
    return false;
  }

  if (bitrate < MIN_BITRATE) {
    return false;
  }

  if (bitrate > MAX_BITRATE) {
    return false;
  }

  return true;
}

function readBitrate(): number {
  const stored = Number(localStorage.getItem(keyBitrate) ?? defaultBitrate);

  // Fallback on invalid
  if (!isValidBitrate(stored)) {
    console.log("Invalid screen share bitrate, falling back to default");
    return defaultBitrate;
  }

  return stored;
}
