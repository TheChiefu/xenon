let selectedRoom: string | null = $state(null);

export function getSelectedRoom(): string | null {
  return selectedRoom;
}

export function selectRoom(id: string): void {
  selectedRoom = id;
}
