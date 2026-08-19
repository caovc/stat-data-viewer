import assert from 'node:assert/strict'
import { describe, it } from 'node:test'
import { appendPageRows, shouldFetchMore } from './infiniteScroll.ts'
import type { PageResult } from '../types.ts'

function page(rows: number, offset: number, total = 1000): PageResult {
  return {
    columns: [],
    rows: Array.from({ length: rows }, () => []),
    offset,
    pageSize: rows,
    totalRows: total,
  }
}

describe('shouldFetchMore', () => {
  const base = {
    mode: 'infinite' as const,
    loaded: 300,
    total: 1200,
    lastVisibleIndex: 280,
    threshold: 24,
    busy: false,
    error: false,
  }

  it('loads the next batch when the window reaches the loaded tail', () => {
    assert.equal(shouldFetchMore(base), true)
  })

  it('does not load while the window is still in the middle', () => {
    assert.equal(shouldFetchMore({ ...base, lastVisibleIndex: 40 }), false)
  })

  it('does not load in page mode', () => {
    assert.equal(shouldFetchMore({ ...base, mode: 'page' }), false)
  })

  it('stops at the last row', () => {
    assert.equal(shouldFetchMore({ ...base, loaded: 1200, lastVisibleIndex: 1190 }), false)
  })

  it('keeps filling when every loaded row is still on screen', () => {
    assert.equal(shouldFetchMore({ ...base, loaded: 80, lastVisibleIndex: 79 }), true)
  })
})

describe('appendPageRows', () => {
  it('concatenates the next contiguous batch', () => {
    const next = appendPageRows(page(300, 0), page(300, 300, 900))
    assert.equal(next.rows.length, 600)
    assert.equal(next.offset, 0)
    assert.equal(next.totalRows, 900)
  })

  it('ignores a batch that does not continue the current rows', () => {
    const current = page(300, 0)
    assert.equal(appendPageRows(current, page(300, 600)), current)
  })
})
