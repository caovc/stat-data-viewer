export const GRID_ROW_HEIGHT = 32

export type VirtualRangeItem = {
  start: number
  end: number
}

/** Spacer rows so a virtualized table can scroll through every item. */
export function virtualRowPads(
  items: readonly VirtualRangeItem[],
  totalSize: number,
): { top: number; bottom: number } {
  const first = items[0]
  const last = items.at(-1)
  if (!first || !last) return { top: 0, bottom: 0 }
  return {
    top: first.start,
    bottom: Math.max(0, totalSize - last.end),
  }
}
