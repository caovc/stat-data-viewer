import assert from 'node:assert/strict'
import { describe, it } from 'node:test'
import type { SqlTable } from './catalog.ts'
import { columnHoverMarkdown, shouldHoverTable, tableHoverMarkdown } from './sqlDocs.ts'

const table: SqlTable = {
  name: 'adcm',
  title: 'adcm.sas7bdat',
  path: '/study/raw/adcm.sas7bdat',
  fileLabel: null,
  rowCount: 9772,
  varCount: 67,
  columns: [
    {
      name: 'ACTARM',
      table: 'adcm',
      label: 'Description of Actual Arm',
      storageType: 'string',
      displayFormat: null,
      measure: null,
      missingRules: null,
    },
  ],
}

describe('tableHoverMarkdown', () => {
  it('shows the full dataset path instead of the basename', () => {
    const text = tableHoverMarkdown(table)
    assert.match(text, /\/study\/raw\/adcm\.sas7bdat/)
    assert.doesNotMatch(text, /Source: adcm\.sas7bdat/)
  })
})

describe('columnHoverMarkdown', () => {
  it('includes the parent table path', () => {
    const text = columnHoverMarkdown(table.columns[0], table)
    assert.match(text, /ACTARM/)
    assert.match(text, /\/study\/raw\/adcm\.sas7bdat/)
  })
})

describe('shouldHoverTable', () => {
  it('hides table hover while completing columns after a dot', () => {
    assert.equal(shouldHoverTable('.'), false)
    assert.equal(shouldHoverTable('.ACT'), false)
    assert.equal(shouldHoverTable(''), true)
    assert.equal(shouldHoverTable(' WHERE'), true)
  })
})
