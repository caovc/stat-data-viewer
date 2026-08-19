import assert from 'node:assert/strict'
import { describe, it } from 'node:test'
import { GRID_ROW_HEIGHT, virtualRowPads, virtualWindowUncovered } from './virtualTable.ts'

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

describe('virtualWindowUncovered', () => {
  const headerSize = GRID_ROW_HEIGHT
  const totalSize = 300 * GRID_ROW_HEIGHT
  const items = Array.from({ length: 20 }, (_, i) => {
    const index = 40 + i
    return { start: index * GRID_ROW_HEIGHT, end: (index + 1) * GRID_ROW_HEIGHT }
  })

  it('reports a gap when the painted rows miss the viewport', () => {
    assert.equal(virtualWindowUncovered({
      items,
      scrollOffset: 0,
      viewportSize: 640,
      headerSize,
      totalSize,
    }), true)
  })

  it('is covered when the painted rows include the visible body', () => {
    assert.equal(virtualWindowUncovered({
      items,
      scrollOffset: 40 * GRID_ROW_HEIGHT + headerSize,
      viewportSize: 640,
      headerSize,
      totalSize,
    }), false)
  })

  it('treats an empty paint list as uncovered', () => {
    assert.equal(virtualWindowUncovered({
      items: [],
      scrollOffset: 200,
      viewportSize: 640,
      headerSize,
      totalSize,
    }), true)
  })
})
