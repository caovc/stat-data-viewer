import assert from 'node:assert/strict'
import { describe, it } from 'node:test'
import type { SqlCatalog } from './catalog.ts'
import { suggestSql } from './sqlSuggest.ts'

function catalog(columnCount: number): SqlCatalog {
  return {
    tables: [
      {
        name: 'adsl',
        title: 'adsl.sav',
        path: '/tmp/adsl.sav',
        fileLabel: null,
        rowCount: 100,
        varCount: columnCount,
        columns: Array.from({ length: columnCount }, (_, index) => ({
          name: index === 0 ? 'USUBJID' : `VAR${index}`,
          table: 'adsl',
          label: null,
          storageType: 'string',
          displayFormat: null,
          measure: null,
          missingRules: null,
        })),
      },
    ],
  }
}

describe('suggestSql', () => {
  it('does not materialize every column on a general keystroke', () => {
    const items = suggestSql({
      prefix: 'SEL',
      sql: 'SEL',
      catalog: catalog(800),
    })
    assert.ok(items.length < 80, `expected a small prefix-filtered set, got ${items.length}`)
    assert.ok(items.some((item) => item.label === 'SELECT'))
    assert.equal(items.some((item) => item.label === 'USUBJID'), false)
  })

  it('suggests only matching columns after a table qualifier', () => {
    const items = suggestSql({
      prefix: 'SELECT adsl.US',
      sql: 'SELECT adsl.US',
      catalog: catalog(800),
    })
    assert.deepEqual(
      items.map((item) => item.label),
      ['USUBJID'],
    )
  })

  it('suggests tables in FROM position without dumping columns', () => {
    const items = suggestSql({
      prefix: 'SELECT * FROM ad',
      sql: 'SELECT * FROM ad',
      catalog: catalog(800),
    })
    assert.deepEqual(
      items.map((item) => item.label),
      ['adsl'],
    )
    assert.match(items[0]?.documentation ?? '', /\/tmp\/adsl\.sav/)
  })
})
