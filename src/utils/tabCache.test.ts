import assert from 'node:assert/strict'
import { describe, it } from 'node:test'
import { overflowTitle, shouldDropInactiveCache, shouldFetchOnActivate } from './tabCache.ts'

describe('shouldFetchOnActivate', () => {
  it('skips a refetch when the tab already has a cached page', () => {
    assert.equal(shouldFetchOnActivate({ rows: [[1]] }), false)
  })

  it('fetches when the tab has never been loaded', () => {
    assert.equal(shouldFetchOnActivate(undefined), true)
    assert.equal(shouldFetchOnActivate(null), true)
  })
})

describe('shouldDropInactiveCache', () => {
  it('keeps the active tab cache so the current grid is not torn down', () => {
    assert.equal(shouldDropInactiveCache(true, true), false)
  })

  it('drops a background tab cache after import completes', () => {
    assert.equal(shouldDropInactiveCache(false, true), true)
    assert.equal(shouldDropInactiveCache(false, false), false)
  })
})

describe('overflowTitle', () => {
  it('returns the text only when the cell is truncated', () => {
    assert.equal(overflowTitle({ scrollWidth: 120, clientWidth: 80 }, 'long'), 'long')
    assert.equal(overflowTitle({ scrollWidth: 80, clientWidth: 80 }, 'short'), '')
    assert.equal(overflowTitle(null, 'x'), '')
  })
})
