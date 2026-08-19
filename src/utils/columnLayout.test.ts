import assert from 'node:assert/strict'
import { describe, it } from 'node:test'
import { moveById, moveItem } from './columnLayout.ts'

describe('moveItem', () => {
  it('moves an item to a new index', () => {
    assert.deepEqual(moveItem(['a', 'b', 'c'], 0, 2), ['b', 'c', 'a'])
    assert.deepEqual(moveItem(['a', 'b', 'c'], 2, 0), ['c', 'a', 'b'])
  })

  it('returns the same list when indexes are invalid', () => {
    const list = ['a', 'b']
    assert.equal(moveItem(list, 0, 0), list)
    assert.equal(moveItem(list, -1, 1), list)
  })
})

describe('moveById', () => {
  it('reorders by id', () => {
    const items = [{ id: 's1' }, { id: 's2' }, { id: 's3' }]
    assert.deepEqual(
      moveById(items, 's1', 's3', (item) => item.id).map((item) => item.id),
      ['s2', 's3', 's1'],
    )
  })
})
