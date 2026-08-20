import assert from 'node:assert/strict'
import { describe, it } from 'node:test'
import type { DataTab, TabView, WorkspaceTab } from '../types.ts'
import {
  fileNameFromPath,
  findDataTabByPath,
  normalizeFilePath,
  omitTableMeta,
  releasedDataTableName,
  sameFilePath,
} from './workspaceTabs.ts'

function dataTab(path: string, tableName: string): DataTab {
  return {
    id: tableName,
    kind: 'data',
    title: fileNameFromPath(path, tableName),
    tableName,
    path,
    jobId: '',
    importing: false,
    progress: 1,
    error: null,
    view: {} as TabView,
  }
}

describe('fileNameFromPath', () => {
  it('keeps only the basename for posix and windows paths', () => {
    assert.equal(fileNameFromPath('/study/a/adsl.sas7bdat'), 'adsl.sas7bdat')
    assert.equal(fileNameFromPath('C:\\study\\b\\adsl.sas7bdat'), 'adsl.sas7bdat')
  })

  it('falls back when the path has no filename', () => {
    assert.equal(fileNameFromPath('/', 'dataset'), 'dataset')
  })
})

describe('sameFilePath', () => {
  it('treats slash variants of the same location as one file', () => {
    assert.equal(sameFilePath('/study/a/adsl.sas7bdat', '/study/a/adsl.sas7bdat'), true)
    assert.equal(sameFilePath('C:\\study\\a\\adsl.sas7bdat', 'C:/study/a/adsl.sas7bdat'), true)
    assert.equal(normalizeFilePath('C:\\study\\a\\adsl.sas7bdat\\'), 'C:/study/a/adsl.sas7bdat')
  })

  it('keeps same-named files from different folders distinct', () => {
    assert.equal(sameFilePath('/study/a/adsl.sas7bdat', '/study/b/adsl.sas7bdat'), false)
  })
})

describe('findDataTabByPath', () => {
  it('reuses the tab for the same path and keeps same-name files as separate tabs', () => {
    const tabs: WorkspaceTab[] = [
      dataTab('/study/a/adsl.sas7bdat', 'adsl'),
      dataTab('/study/b/adsl.sas7bdat', 'adsl_2'),
    ]
    assert.equal(findDataTabByPath(tabs, '/study/a/adsl.sas7bdat')?.tableName, 'adsl')
    assert.equal(findDataTabByPath(tabs, '/study/b/adsl.sas7bdat')?.tableName, 'adsl_2')
    assert.equal(findDataTabByPath(tabs, '/study/c/adsl.sas7bdat'), undefined)
  })
})

describe('releasedDataTableName', () => {
  it('releases a data table only when no remaining tab still uses it', () => {
    const closed = dataTab('/study/a/adsl.sas7bdat', 'adsl')
    const other = dataTab('/study/b/adae.sas7bdat', 'adae')
    assert.equal(releasedDataTableName(closed, [other]), 'adsl')
    assert.equal(releasedDataTableName(closed, [closed, other]), null)
    assert.equal(
      releasedDataTableName({ id: 's1', kind: 'sql', title: 'SQL', sql: '', view: {} as TabView }, []),
      null,
    )
  })
})

describe('omitTableMeta', () => {
  it('drops only the closed table from the SQL catalog cache', () => {
    const next = omitTableMeta({ adsl: 1, adae: 2 }, 'adsl')
    assert.deepEqual(next, { adae: 2 })
    assert.deepEqual(omitTableMeta({ adae: 2 }, 'adsl'), { adae: 2 })
  })
})
