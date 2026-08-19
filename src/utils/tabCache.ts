/** Cached pages should paint immediately; fetch only when this tab has no rows yet. */
export function shouldFetchOnActivate(cachedPage: { rows: unknown[] } | null | undefined): boolean {
  return cachedPage == null
}

/** Drop a background tab's cache when its import finishes so the next visit loads the full table. */
export function shouldDropInactiveCache(isActive: boolean, importComplete: boolean): boolean {
  return importComplete && !isActive
}

/** Native title only when the cell is truncated. */
export function overflowTitle(node: Pick<HTMLElement, 'scrollWidth' | 'clientWidth'> | null, text: string): string {
  if (!node || !text) return ''
  return node.scrollWidth > node.clientWidth + 1 ? text : ''
}
