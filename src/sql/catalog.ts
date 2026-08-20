import type { DatasetMeta, WorkspaceTab } from '../types'

export interface SqlColumn {
  name: string
  table: string
  label: string | null
  storageType: string
  displayFormat: string | null
  measure: string | null
  missingRules: string | null
}

export interface SqlTable {
  name: string
  title: string
  path: string | null
  fileLabel: string | null
  rowCount: number | null
  varCount: number | null
  columns: SqlColumn[]
}

export interface SqlCatalog {
  tables: SqlTable[]
}

const NOT_ALIAS = new Set([
  'anti',
  'asof',
  'cross',
  'except',
  'full',
  'group',
  'having',
  'inner',
  'intersect',
  'join',
  'left',
  'limit',
  'natural',
  'offset',
  'on',
  'order',
  'outer',
  'positional',
  'qualify',
  'right',
  'sample',
  'semi',
  'union',
  'using',
  'where',
  'window',
])

export function quoteIdent(name: string): string {
  return /^[A-Za-z_][A-Za-z0-9_]*$/.test(name) ? name : `"${name.replaceAll('"', '""')}"`
}

export function buildSqlCatalog(
  tabs: Array<Pick<WorkspaceTab, 'kind'> & { tableName?: string; title?: string; path?: string }>,
  metadataByTable: Record<string, DatasetMeta>,
): SqlCatalog {
  const seen = new Set<string>()
  const tables: SqlTable[] = []

  for (const tab of tabs) {
    if (tab.kind !== 'data' || !tab.tableName) continue
    const key = tab.tableName.toLowerCase()
    if (seen.has(key)) continue
    seen.add(key)
    tables.push(
      toSqlTable(
        tab.tableName,
        tab.title ?? tab.tableName,
        tab.path ?? null,
        metadataByTable[tab.tableName],
      ),
    )
  }

  return { tables }
}

export function findTable(catalog: SqlCatalog, name: string): SqlTable | undefined {
  const key = name.toLowerCase()
  return catalog.tables.find((table) => table.name.toLowerCase() === key)
}

export function findColumn(catalog: SqlCatalog, name: string, tableName?: string): SqlColumn | undefined {
  const key = name.toLowerCase()
  const tables = tableName
    ? catalog.tables.filter((table) => table.name.toLowerCase() === tableName.toLowerCase())
    : catalog.tables
  for (const table of tables) {
    const column = table.columns.find((item) => item.name.toLowerCase() === key)
    if (column) return column
  }
  return undefined
}

export function parseTableAliases(sql: string): Map<string, string> {
  const aliases = new Map<string, string>()
  const pattern =
    /\b(?:from|join)\s+(?:"([^"]+)"|([A-Za-z_][\w]*))(?:\s+(?:as\s+)?(?:"([^"]+)"|([A-Za-z_][\w]*)))?/gi
  for (const match of sql.matchAll(pattern)) {
    const table = match[1] ?? match[2]
    if (!table) continue
    aliases.set(table.toLowerCase(), table)
    const alias = match[3] ?? match[4]
    if (alias && !NOT_ALIAS.has(alias.toLowerCase())) {
      aliases.set(alias.toLowerCase(), table)
    }
  }
  return aliases
}

function toSqlTable(name: string, title: string, path: string | null, meta?: DatasetMeta): SqlTable {
  return {
    name,
    title,
    path: path || meta?.sourcePath || null,
    fileLabel: meta?.fileLabel ?? null,
    rowCount: meta?.rowCount ?? null,
    varCount: meta?.varCount ?? meta?.variables.length ?? null,
    columns: (meta?.variables ?? []).map((item) => ({
      name: item.name,
      table: name,
      label: item.label,
      storageType: item.storageType,
      displayFormat: item.displayFormat,
      measure: item.measure,
      missingRules: item.missingRules,
    })),
  }
}
