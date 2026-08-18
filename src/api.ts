import { invoke } from '@tauri-apps/api/core'
import type {
  DatasetMeta,
  DistinctResult,
  FilterGroup,
  OpenResult,
  PageResult,
  SortSpec,
} from './types'

export const DATA_FILTERS = [
  {
    key: 'files.all' as const,
    name: 'Statistical data',
    extensions: ['sas7bdat', 'xpt', 'sav', 'zsav', 'por', 'dta'],
  },
  { key: 'files.sas' as const, name: 'SAS dataset', extensions: ['sas7bdat'] },
  { key: 'files.xpt' as const, name: 'SAS transport', extensions: ['xpt'] },
  { key: 'files.spss' as const, name: 'SPSS', extensions: ['sav', 'zsav', 'por'] },
  { key: 'files.stata' as const, name: 'Stata', extensions: ['dta'] },
]

export const CATALOG_FILTERS = [
  { key: 'files.catalog' as const, name: 'SAS catalog', extensions: ['sas7bcat'] },
]

export function openDataset(args: {
  path: string
  encoding?: string
  format?: string
  catalogPath?: string
}) {
  return invoke<OpenResult>('open_dataset', { args })
}

export function reimport(
  table: string,
  args: { path: string; encoding?: string; format?: string; catalogPath?: string },
) {
  return invoke<OpenResult>('reimport', { table, args })
}

export function cancelImport(jobId: string) {
  return invoke<void>('cancel_import', { jobId })
}

export function queryPage(args: {
  table: string
  offset: number
  pageSize?: number
  sorts?: SortSpec[]
  filters?: FilterGroup
  hidden?: string[]
}) {
  return invoke<PageResult>('query_page', { args })
}

export function runSql(args: { sql: string; offset: number; pageSize?: number }) {
  return invoke<PageResult>('run_sql', { args })
}

export function exportResult(args: {
  path: string
  format: string
  table?: string
  sql?: string
}) {
  return invoke<void>('export', { args })
}

export function getMetadata(table: string) {
  return invoke<DatasetMeta>('get_metadata', { table })
}

export function columnDistinct(args: { table: string; column: string; limit?: number }) {
  return invoke<DistinctResult>('column_distinct', { args })
}
