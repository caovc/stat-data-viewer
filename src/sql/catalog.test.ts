import assert from 'node:assert/strict'
import { describe, it } from 'node:test'
import type { DatasetMeta, DataTab, TabView } from '../types.ts'
import { buildSqlCatalog } from './catalog.ts'

function dataTab(tableName: string, title = tableName): DataTab {
  return {
    id: tableName,
    kind: 'data',
    title,
    tableName,
    path: `/tmp/${title}`,
    jobId: '',
    importing: false,
    progress: 1,
    error: null,
    view: {} as TabView,
  }
}

function meta(tableName: string): DatasetMeta {
  return {
    tableName,
    sourcePath: `/tmp/${tableName}.sav`,
    fileFormat: 'sav',
    encoding: null,
    fileLabel: null,
    formatVersion: null,
    rowCount: 1,
    varCount: 1,
    catalogPath: null,
    importComplete: true,
    variables: [
      {
        index: 0,
        name: 'id',
        label: null,
        storageType: 'int32',
        displayFormat: null,
        measure: null,
        displayWidth: null,
        decimals: null,
        missingRules: null,
        labelSet: null,
      },
    ],
    valueLabels: [],
  }
}

describe('buildSqlCatalog', () => {
  it('lists only tables that still have an open data tab', () => {
    const catalog = buildSqlCatalog(
      [dataTab('adsl', 'adsl.sav')],
      { adsl: meta('adsl'), adae: meta('adae') },
    )
    assert.deepEqual(
      catalog.tables.map((table) => table.name),
      ['adsl'],
    )
    assert.equal(catalog.tables[0]?.path, '/tmp/adsl.sav')
  })

  it('keeps a closed table out of completions even if metadata was left behind', () => {
    const catalog = buildSqlCatalog([], { adsl: meta('adsl') })
    assert.deepEqual(catalog.tables, [])
  })
})
