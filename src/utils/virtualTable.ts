export const GRID_ROW_HEIGHT = 32
export const GRID_OVERSCAN = 10

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

/** True when the painted virtual rows do not cover the visible body. */
export function virtualWindowUncovered(input: {
  items: readonly VirtualRangeItem[]
  scrollOffset: number
  viewportSize: number
  headerSize: number
  totalSize: number
}): boolean {
  if (input.viewportSize <= 0 || input.totalSize <= 0) return false
  if (input.items.length === 0) return true
  const body = Math.max(0, input.viewportSize - input.headerSize)
  const viewStart = Math.max(0, input.scrollOffset - input.headerSize)
  const viewEnd = Math.min(input.totalSize, viewStart + body)
  if (viewEnd <= viewStart) return false
  const start = input.items[0]!.start
  const end = input.items.at(-1)!.end
  const slack = GRID_ROW_HEIGHT
  return start > viewStart + slack || end < viewEnd - slack
}
