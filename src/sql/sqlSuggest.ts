import { findTable, parseTableAliases, type SqlCatalog, type SqlColumn, type SqlTable } from './catalog.ts'
import { DUCKDB_FUNCTIONS, DUCKDB_KEYWORDS, DUCKDB_TYPES } from './duckdbLanguage.ts'
import { columnHoverMarkdown, tableHoverMarkdown } from './sqlDocs.ts'

export const TABLE_CONTEXT = /\b(from|join|into|update|table|copy)\s+["\w]*$/i

export type SqlSuggestKind = 'table' | 'column' | 'function' | 'keyword' | 'type' | 'snippet'

export interface SqlSuggestItem {
  kind: SqlSuggestKind
  label: string
  insertText: string
  detail: string
  documentation?: string
  sortText: string
  snippet?: boolean
}

export interface SqlSuggestInput {
  prefix: string
  sql: string
  catalog: SqlCatalog
}

const FUNCTION_ITEMS: SqlSuggestItem[] = DUCKDB_FUNCTIONS.map((fn) => ({
  kind: 'function',
  label: fn.name,
  insertText: `${fn.name}($0)`,
  detail: fn.signature,
  documentation: fn.detail,
  sortText: `3_${fn.name}`,
  snippet: true,
}))

const KEYWORD_ITEMS: SqlSuggestItem[] = DUCKDB_KEYWORDS.map((keyword) => ({
  kind: 'keyword',
  label: keyword,
  insertText: keyword,
  detail: 'keyword',
  sortText: `4_${keyword}`,
}))

const TYPE_ITEMS: SqlSuggestItem[] = DUCKDB_TYPES.map((type) => ({
  kind: 'type',
  label: type,
  insertText: type,
  detail: 'type',
  sortText: `5_${type}`,
}))

const SNIPPET_ITEMS: SqlSuggestItem[] = [
  {
    kind: 'snippet',
    label: 'sel',
    insertText: 'SELECT ${1:*} FROM ${2:table}',
    detail: 'SELECT … FROM',
    sortText: '2_sel',
    snippet: true,
  },
  {
    kind: 'snippet',
    label: 'join',
    insertText: 'JOIN ${1:table} USING (${2:USUBJID})',
    detail: 'JOIN … USING',
    sortText: '2_join',
    snippet: true,
  },
  {
    kind: 'snippet',
    label: 'ljoin',
    insertText: 'LEFT JOIN ${1:table} ON ${2:a}.${3:col} = ${4:b}.${5:col}',
    detail: 'LEFT JOIN … ON',
    sortText: '2_ljoin',
    snippet: true,
  },
]

export const DUCKDB_FUNCTION_BY_NAME = new Map(
  DUCKDB_FUNCTIONS.map((item) => [item.name.toLowerCase(), item]),
)

export function memberAccess(prefix: string): { table: string } | null {
  const match = /(?:"([^"]+)"|([A-Za-z_][\w]*))\s*\.\s*(?:"[^"]*"|[\w]*)$/.exec(prefix)
  if (!match) return null
  return { table: match[1] ?? match[2] }
}

export function currentSuggestWord(prefix: string): string {
  const quoted = /"([^"]*)$/.exec(prefix)
  if (quoted) return quoted[1]
  return /[A-Za-z_][\w]*$/.exec(prefix)?.[0] ?? ''
}

function matchesWord(label: string, word: string): boolean {
  if (!word) return true
  return label.toLowerCase().startsWith(word.toLowerCase())
}

function tableItem(table: SqlTable, bucket: string): SqlSuggestItem {
  const extra = [
    table.rowCount != null ? `${table.rowCount.toLocaleString()} rows` : null,
    table.varCount != null ? `${table.varCount} vars` : null,
  ]
    .filter(Boolean)
    .join(' · ')
  return {
    kind: 'table',
    label: table.name,
    insertText: table.name,
    detail: extra ? `table · ${extra}` : 'table',
    documentation: tableHoverMarkdown(table),
    sortText: `${bucket}_${table.name}`,
  }
}

function columnItem(column: SqlColumn, table: SqlTable): SqlSuggestItem {
  return {
    kind: 'column',
    label: column.name,
    insertText: column.name,
    detail: [column.storageType, column.label].filter(Boolean).join(' · '),
    documentation: columnHoverMarkdown(column, table),
    sortText: `0_${column.name}`,
  }
}

function columnsForTable(catalog: SqlCatalog, tableName: string, word: string): SqlSuggestItem[] {
  const table = findTable(catalog, tableName)
  if (!table) return []
  return table.columns.filter((column) => matchesWord(column.name, word)).map((column) => columnItem(column, table))
}

export function suggestSql(input: SqlSuggestInput): SqlSuggestItem[] {
  const word = currentSuggestWord(input.prefix)
  const member = memberAccess(input.prefix)
  if (member) {
    const aliases = parseTableAliases(input.sql)
    const tableName = aliases.get(member.table.toLowerCase()) ?? member.table
    return columnsForTable(input.catalog, tableName, word)
  }

  if (TABLE_CONTEXT.test(input.prefix)) {
    return input.catalog.tables.filter((table) => matchesWord(table.name, word)).map((table) => tableItem(table, '0'))
  }

  if (!word) return []

  const aliases = parseTableAliases(input.sql)
  const scoped = new Set<string>()
  const items: SqlSuggestItem[] = []
  for (const tableName of aliases.values()) {
    const key = tableName.toLowerCase()
    if (scoped.has(key)) continue
    scoped.add(key)
    items.push(...columnsForTable(input.catalog, tableName, word))
  }
  for (const table of input.catalog.tables) {
    if (matchesWord(table.name, word)) items.push(tableItem(table, '1'))
  }
  for (const item of SNIPPET_ITEMS) {
    if (matchesWord(item.label, word)) items.push(item)
  }
  for (const item of FUNCTION_ITEMS) {
    if (matchesWord(item.label, word)) items.push(item)
  }
  for (const item of KEYWORD_ITEMS) {
    if (matchesWord(item.label, word)) items.push(item)
  }
  for (const item of TYPE_ITEMS) {
    if (matchesWord(item.label, word)) items.push(item)
  }
  return items
}
