import type { editor, IRange, languages, Position } from './monaco'
import { languages as monacoLanguages } from './monaco'
import {
  findColumn,
  findTable,
  parseTableAliases,
  quoteIdent,
  type SqlCatalog,
  type SqlColumn,
  type SqlTable,
} from './catalog'
import { DUCKDB_FUNCTIONS, DUCKDB_KEYWORDS, DUCKDB_TYPES } from './duckdbLanguage'

export type CatalogGetter = () => SqlCatalog

const TABLE_CONTEXT = /\b(from|join|into|update|table|copy)\s+["\w]*$/i

export function createCompletionProvider(
  getCatalog: CatalogGetter,
): languages.CompletionItemProvider {
  return {
    triggerCharacters: ['.', ' ', '"'],
    provideCompletionItems(model, position) {
      const catalog = getCatalog()
      const range = currentWordRange(model, position)
      const prefix = textBefore(model, position)
      const member = memberAccess(prefix)
      const aliases = parseTableAliases(model.getValue())
      const Kind = monacoLanguages.CompletionItemKind
      const snippet = monacoLanguages.CompletionItemInsertTextRule.InsertAsSnippet
      const suggestions: languages.CompletionItem[] = []

      if (member) {
        const tableName = aliases.get(member.table.toLowerCase()) ?? member.table
        const table = findTable(catalog, tableName)
        for (const column of table?.columns ?? []) {
          suggestions.push(columnItem(column, range, Kind.Field, '0'))
        }
        return { suggestions }
      }

      const wantTables = TABLE_CONTEXT.test(prefix)
      const wantColumns = !wantTables

      if (wantTables || !wantColumns) {
        for (const table of catalog.tables) {
          suggestions.push(tableItem(table, range, Kind.Struct, wantTables ? '0' : '1'))
        }
      }

      if (wantColumns) {
        const seen = new Set<string>()
        for (const table of catalog.tables) {
          for (const column of table.columns) {
            const key = column.name.toLowerCase()
            if (seen.has(key)) continue
            seen.add(key)
            suggestions.push(columnItem(column, range, Kind.Field, '0'))
          }
        }
      }

      if (!wantTables) {
        for (const fn of DUCKDB_FUNCTIONS) {
          suggestions.push({
            label: fn.name,
            kind: Kind.Function,
            insertText: `${fn.name}($0)`,
            insertTextRules: snippet,
            detail: fn.signature,
            documentation: { value: fn.detail },
            range,
            sortText: `3_${fn.name}`,
          })
        }
        for (const keyword of DUCKDB_KEYWORDS) {
          suggestions.push({
            label: keyword,
            kind: Kind.Keyword,
            insertText: keyword,
            detail: 'keyword',
            range,
            sortText: `4_${keyword}`,
          })
        }
        for (const type of DUCKDB_TYPES) {
          suggestions.push({
            label: type,
            kind: Kind.TypeParameter,
            insertText: type,
            detail: 'type',
            range,
            sortText: `5_${type}`,
          })
        }
        suggestions.push(
          {
            label: 'sel',
            kind: Kind.Snippet,
            insertText: 'SELECT ${1:*} FROM ${2:table}',
            insertTextRules: snippet,
            detail: 'SELECT … FROM',
            range,
            sortText: '2_sel',
          },
          {
            label: 'join',
            kind: Kind.Snippet,
            insertText: 'JOIN ${1:table} USING (${2:USUBJID})',
            insertTextRules: snippet,
            detail: 'JOIN … USING',
            range,
            sortText: '2_join',
          },
          {
            label: 'ljoin',
            kind: Kind.Snippet,
            insertText: 'LEFT JOIN ${1:table} ON ${2:a}.${3:col} = ${4:b}.${5:col}',
            insertTextRules: snippet,
            detail: 'LEFT JOIN … ON',
            range,
            sortText: '2_ljoin',
          },
        )
      }

      return { suggestions }
    },
  }
}

export function createHoverProvider(getCatalog: CatalogGetter): languages.HoverProvider {
  return {
    provideHover(model, position) {
      const token = identAt(model, position)
      if (!token) return null
      const catalog = getCatalog()
      const aliases = parseTableAliases(model.getValue())
      const qualifier = token.qualifier
        ? aliases.get(token.qualifier.toLowerCase()) ?? token.qualifier
        : undefined
      const table = findTable(catalog, token.name)
      if (table && !qualifier) {
        return { range: token.range, contents: [{ value: tableMarkdown(table) }] }
      }
      const column = findColumn(catalog, token.name, qualifier)
      if (column) {
        return { range: token.range, contents: [{ value: columnMarkdown(column) }] }
      }
      const fn = DUCKDB_FUNCTIONS.find((item) => item.name.toLowerCase() === token.name.toLowerCase())
      if (fn) {
        return {
          range: token.range,
          contents: [{ value: `**${fn.signature}**\n\n${fn.detail}` }],
        }
      }
      return null
    },
  }
}

export function createSignatureHelpProvider(): languages.SignatureHelpProvider {
  return {
    signatureHelpTriggerCharacters: ['(', ','],
    provideSignatureHelp(model, position) {
      const line = model.getLineContent(position.lineNumber).slice(0, position.column - 1)
      const match = /([A-Za-z_][\w]*)\s*\([^()]*$/.exec(line)
      if (!match) return null
      const fn = DUCKDB_FUNCTIONS.find((item) => item.name.toLowerCase() === match[1].toLowerCase())
      if (!fn) return null
      return {
        value: {
          signatures: [
            {
              label: fn.signature,
              documentation: fn.detail,
              parameters: [],
            },
          ],
          activeSignature: 0,
          activeParameter: 0,
        },
        dispose() {},
      }
    },
  }
}

function tableItem(
  table: SqlTable,
  range: IRange,
  kind: languages.CompletionItemKind,
  bucket: string,
): languages.CompletionItem {
  const extra = [
    table.rowCount != null ? `${table.rowCount.toLocaleString()} rows` : null,
    table.varCount != null ? `${table.varCount} vars` : null,
  ]
    .filter(Boolean)
    .join(' · ')
  return {
    label: table.name,
    kind,
    insertText: quoteIdent(table.name),
    detail: extra ? `table · ${extra}` : 'table',
    documentation: { value: tableMarkdown(table) },
    range,
    sortText: `${bucket}_${table.name}`,
  }
}

function columnItem(
  column: SqlColumn,
  range: IRange,
  kind: languages.CompletionItemKind,
  bucket: string,
): languages.CompletionItem {
  const bits = [column.storageType, column.label].filter(Boolean)
  return {
    label: column.name,
    kind,
    insertText: quoteIdent(column.name),
    detail: bits.join(' · '),
    documentation: { value: columnMarkdown(column) },
    range,
    sortText: `${bucket}_${column.name}`,
  }
}

function tableMarkdown(table: SqlTable): string {
  const lines = [`**${table.name}**`]
  if (table.fileLabel) lines.push(table.fileLabel)
  const bits = [
    table.rowCount != null ? `${table.rowCount.toLocaleString()} rows` : null,
    table.varCount != null ? `${table.varCount} variables` : null,
  ].filter(Boolean)
  if (bits.length) lines.push(bits.join(' · '))
  if (table.title && table.title !== table.name) lines.push(`Source: ${table.title}`)
  if (table.columns.length) {
    const preview = table.columns
      .slice(0, 8)
      .map((column) => `\`${column.name}\``)
      .join(', ')
    const more = table.columns.length > 8 ? `, …` : ''
    lines.push(preview + more)
  }
  return lines.join('\n\n')
}

function columnMarkdown(column: SqlColumn): string {
  const lines = [`**${column.name}**`]
  if (column.label) lines.push(column.label)
  const bits = [`\`${column.storageType}\``]
  if (column.displayFormat) bits.push(`format ${column.displayFormat}`)
  if (column.measure) bits.push(column.measure)
  lines.push(bits.join(' · '))
  lines.push(`Table: \`${column.table}\``)
  if (column.missingRules) lines.push(`Missing: ${column.missingRules}`)
  return lines.join('\n\n')
}

function currentWordRange(model: editor.ITextModel, position: Position): IRange {
  const word = model.getWordUntilPosition(position)
  return {
    startLineNumber: position.lineNumber,
    endLineNumber: position.lineNumber,
    startColumn: word.startColumn,
    endColumn: word.endColumn,
  }
}

function textBefore(model: editor.ITextModel, position: Position): string {
  return model.getValueInRange({
    startLineNumber: Math.max(1, position.lineNumber - 8),
    startColumn: 1,
    endLineNumber: position.lineNumber,
    endColumn: position.column,
  })
}

function memberAccess(prefix: string): { table: string } | null {
  const match = /(?:"([^"]+)"|([A-Za-z_][\w]*))\s*\.\s*(?:"[^"]*"|[\w]*)$/.exec(prefix)
  if (!match) return null
  return { table: match[1] ?? match[2] }
}

function identAt(
  model: editor.ITextModel,
  position: Position,
): { name: string; qualifier?: string; range: IRange } | null {
  const line = model.getLineContent(position.lineNumber)
  const index = position.column - 1
  const quoted = quotedIdentAt(line, index, position.lineNumber)
  if (quoted) {
    const before = line.slice(0, quoted.start - 1)
    const qual = /(?:"([^"]+)"|([A-Za-z_][\w]*))\s*\.\s*$/.exec(before)
    return {
      name: quoted.name,
      qualifier: qual?.[1] ?? qual?.[2],
      range: quoted.range,
    }
  }
  const word = model.getWordAtPosition(position)
  if (!word) return null
  const before = line.slice(0, word.startColumn - 1)
  const qual = /(?:"([^"]+)"|([A-Za-z_][\w]*))\s*\.\s*$/.exec(before)
  return {
    name: word.word,
    qualifier: qual?.[1] ?? qual?.[2],
    range: {
      startLineNumber: position.lineNumber,
      endLineNumber: position.lineNumber,
      startColumn: word.startColumn,
      endColumn: word.endColumn,
    },
  }
}

function quotedIdentAt(line: string, index: number, lineNumber: number) {
  const start = line.lastIndexOf('"', index)
  if (start < 0) return null
  const end = line.indexOf('"', start + 1)
  if (end < 0 || index > end) return null
  return {
    name: line.slice(start + 1, end),
    start,
    range: {
      startLineNumber: lineNumber,
      endLineNumber: lineNumber,
      startColumn: start + 1,
      endColumn: end + 2,
    },
  }
}
