import type { ColumnPin } from '../types'

export const ROW_HEAD_WIDTH = 56
export const DEFAULT_COLUMN_WIDTH = 160
export const MIN_COLUMN_WIDTH = 72
export const MAX_COLUMN_WIDTH = 720

export function columnWidthOf(name: string, widths: Record<string, number>): number {
  return widths[name] ?? (name === '_row' ? ROW_HEAD_WIDTH : DEFAULT_COLUMN_WIDTH)
}

export function mergeColumnOrder(order: string[], names: string[]): string[] {
  const known = new Set(names)
  const kept = order.filter((name) => known.has(name))
  const keptSet = new Set(kept)
  return [...kept, ...names.filter((name) => !keptSet.has(name))]
}

export function moveItem<T>(list: T[], from: number, to: number): T[] {
  if (from === to || from < 0 || to < 0 || from >= list.length || to >= list.length) {
    return list
  }
  const next = [...list]
  const [item] = next.splice(from, 1)
  if (item === undefined) return list
  next.splice(to, 0, item)
  return next
}

export function moveById<T>(list: T[], fromId: string, toId: string, idOf: (item: T) => string): T[] {
  return moveItem(
    list,
    list.findIndex((item) => idOf(item) === fromId),
    list.findIndex((item) => idOf(item) === toId),
  )
}

export function displayColumnNames(
  order: string[],
  pinnedStart: string[],
  pinnedEnd: string[],
): string[] {
  const start = pinnedStart.filter((name) => order.includes(name))
  const end = pinnedEnd.filter((name) => order.includes(name))
  const pinned = new Set([...start, ...end])
  return [...start, ...order.filter((name) => !pinned.has(name)), ...end]
}

export function pinOf(name: string, pinnedStart: string[], pinnedEnd: string[]): ColumnPin {
  if (pinnedStart.includes(name)) return 'start'
  if (pinnedEnd.includes(name)) return 'end'
  return null
}

export function visiblePinned(pinned: string[], displayed: Iterable<string>): string[] {
  const shown = new Set(displayed)
  return pinned.filter((name) => shown.has(name))
}

export function pinStartOffset(
  name: string,
  pinnedStart: string[],
  sizeOf: (name: string) => number,
  leadWidth = 0,
): number | null {
  const index = pinnedStart.indexOf(name)
  if (index < 0) return null
  return leadWidth + pinnedStart.slice(0, index).reduce((sum, prev) => sum + sizeOf(prev), 0)
}

export function pinEndOffset(
  name: string,
  pinnedEnd: string[],
  sizeOf: (name: string) => number,
): number | null {
  const index = pinnedEnd.indexOf(name)
  if (index < 0) return null
  return pinnedEnd.slice(index + 1).reduce((sum, next) => sum + sizeOf(next), 0)
}

export function pinStickyStyle(
  name: string,
  pinnedStart: string[],
  pinnedEnd: string[],
  sizeOf: (name: string) => number,
  rowHeadId = '_row',
): Record<string, string> {
  const width = sizeOf(name)
  const box: Record<string, string> = {
    width: `${width}px`,
    minWidth: `${width}px`,
    maxWidth: `${width}px`,
  }
  if (name === rowHeadId) {
    box.left = '0px'
    return box
  }
  const left = pinStartOffset(name, pinnedStart, sizeOf, sizeOf(rowHeadId))
  if (left != null) {
    box.left = `${left}px`
    return box
  }
  const right = pinEndOffset(name, pinnedEnd, sizeOf)
  if (right != null) {
    box.right = `${right}px`
    return box
  }
  return box
}

export function nextPinList(
  name: string,
  pin: ColumnPin,
  pinnedStart: string[],
  pinnedEnd: string[],
): { pinnedStart: string[]; pinnedEnd: string[] } {
  const start = pinnedStart.filter((item) => item !== name)
  const end = pinnedEnd.filter((item) => item !== name)
  if (pin === 'start') start.push(name)
  if (pin === 'end') end.push(name)
  return { pinnedStart: start, pinnedEnd: end }
}

export function isDefaultColumnLayout(input: {
  names: string[]
  order: string[]
  hidden: string[]
  pinnedStart: string[]
  pinnedEnd: string[]
  widths?: Record<string, number>
}): boolean {
  if (input.hidden.length > 0 || input.pinnedStart.length > 0 || input.pinnedEnd.length > 0) {
    return false
  }
  if (input.widths && Object.keys(input.widths).length > 0) {
    return false
  }
  if (input.order.length === 0) return true
  return input.order.length === input.names.length
    && input.order.every((name, index) => name === input.names[index])
}
