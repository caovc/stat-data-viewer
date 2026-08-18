import assert from 'node:assert/strict'
import { describe, it } from 'node:test'
import { GRID_ROW_HEIGHT, virtualRowPads } from './virtualTable.ts'

describe('virtualRowPads', () => {
  it('reserves the full page height around the visible window', () => {
    const pageSize = 300
    const totalSize = pageSize * GRID_ROW_HEIGHT
    const windowStart = 50
    const windowCount = 20
    const items = Array.from({ length: windowCount }, (_, i) => {
      const index = windowStart + i
      return { start: index * GRID_ROW_HEIGHT, end: (index + 1) * GRID_ROW_HEIGHT }
    })

    const pads = virtualRowPads(items, totalSize)

    assert.equal(pads.top, windowStart * GRID_ROW_HEIGHT)
    assert.equal(
      pads.top + windowCount * GRID_ROW_HEIGHT + pads.bottom,
      totalSize,
    )
  })

  it('uses zero spacers when nothing is virtualized', () => {
    assert.deepEqual(virtualRowPads([], 9600), { top: 0, bottom: 0 })
  })
})
