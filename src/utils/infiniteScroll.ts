import type { PageResult, ScrollMode } from '../types'

export const PAGE_SIZE_OPTIONS = [100, 300, 500, 1000] as const
export const LOAD_MORE_THRESHOLD = 24

export function shouldFetchMore(input: {
  mode: ScrollMode
  loaded: number
  total: number
  lastVisibleIndex: number | null
  threshold: number
  busy: boolean
  error: boolean
}): boolean {
  if (input.mode !== 'infinite' || input.busy || input.error) return false
  if (input.loaded <= 0 || input.loaded >= input.total) return false
  if (input.lastVisibleIndex == null) return false
  return input.lastVisibleIndex >= input.loaded - input.threshold
}

export function appendPageRows(current: PageResult, next: PageResult): PageResult {
  if (next.offset !== current.rows.length) return current
  return {
    ...next,
    rows: [...current.rows, ...next.rows],
    offset: 0,
  }
}
