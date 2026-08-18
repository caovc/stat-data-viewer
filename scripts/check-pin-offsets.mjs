import assert from 'node:assert/strict'
import {
  pinEndOffset,
  pinStartOffset,
  pinStickyStyle,
  visiblePinned,
} from '../src/utils/columnLayout.ts'

const sizeOf = (name) => ({ _row: 56, SUBJID: 160, AGE: 120, SEX: 80 }[name] ?? 160)

const displayed = ['SUBJID', 'AGE', 'SEX']
const pinnedStart = visiblePinned(['SUBJID', 'HIDDEN', 'AGE'], displayed)
const pinnedEnd = visiblePinned(['SEX', 'GONE'], displayed)

assert.deepEqual(pinnedStart, ['SUBJID', 'AGE'], 'hidden pinned columns must not affect start offsets')
assert.deepEqual(pinnedEnd, ['SEX'], 'hidden pinned columns must not affect end offsets')
assert.equal(pinStartOffset('SUBJID', pinnedStart, sizeOf, sizeOf('_row')), 56)
assert.equal(pinStartOffset('AGE', pinnedStart, sizeOf, sizeOf('_row')), 216)
assert.equal(pinStartOffset('SEX', pinnedStart, sizeOf, sizeOf('_row')), null)
assert.equal(pinEndOffset('SEX', pinnedEnd, sizeOf), 0)
assert.equal(pinEndOffset('AGE', pinnedEnd, sizeOf), null)

const startStyle = pinStickyStyle('AGE', pinnedStart, pinnedEnd, sizeOf)
assert.equal(startStyle.left, '216px')
assert.equal(startStyle.right, undefined)

const endStyle = pinStickyStyle('SEX', pinnedStart, pinnedEnd, sizeOf)
assert.equal(endStyle.right, '0px')
assert.equal(endStyle.left, undefined)

const rowStyle = pinStickyStyle('_row', pinnedStart, pinnedEnd, sizeOf)
assert.equal(rowStyle.left, '0px')

console.log('pin offsets ok')
