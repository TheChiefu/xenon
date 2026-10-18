// One clickable entry in the context menu
export type MenuItem = { label: string, action: () => void };

// Context menu position, undefined when closed
let menuPos: { x: number, y: number } | undefined = $state();

// Entries shown while the menu is open
let menuItems: MenuItem[] = $state([]);

export function getPos(): { x: number, y: number } | undefined {
  return menuPos;
}

export function getItems(): MenuItem[] {
  return menuItems;
}

// Opens the menu at the pointer with the given entries. Stops the event from
// reaching parent elements, so an outer handler can't replace these entries
export function open(event: MouseEvent, items: MenuItem[]): void {
  event.preventDefault();
  event.stopPropagation();
  menuPos = { x: event.clientX, y: event.clientY };
  menuItems = items;
}

export function close(): void {
  menuPos = undefined;
}

// Closes the menu, then runs the chosen entry
export function chooseItem(item: MenuItem): void {
  close();
  item.action();
}
