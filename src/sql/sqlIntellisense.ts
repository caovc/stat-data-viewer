import type { editor, IRange, languages, Position } from './monaco'
import { languages as monacoLanguages } from './monaco'
import {
  findColumn,
  findTable,
  parseTableAliases,
  quoteIdent,
  type SqlCatalog,
} from './catalog'
import { DUCKDB_FUNCTION_BY_NAME, suggestSql } from './sqlSuggest'
import { columnHoverMarkdown, shouldHoverTable, tableHoverMarkdown } from './sqlDocs'

export type CatalogGetter = () => SqlCatalog

const KIND: Record<string, languages.CompletionItemKind> = {
  table: monacoLanguages.CompletionItemKind.Struct,
  column: monacoLanguages.CompletionItemKind.Field,
  function: monacoLanguages.CompletionItemKind.Function,
  keyword: monacoLanguages.CompletionItemKind.Keyword,
  type: monacoLanguages.CompletionItemKind.TypeParameter,
  snippet: monacoLanguages.CompletionItemKind.Snippet,
}

const aliasCache = new WeakMap<editor.ITextModel, { version: number; aliases: Map<string, string> }>()

function aliasesFor(model: editor.ITextModel): Map<string, string> {
  const version = model.getVersionId()
  const hit = aliasCache.get(model)
  if (hit && hit.version === version) return hit.aliases
  const aliases = parseTableAliases(model.getValue())
  aliasCache.set(model, { version, aliases })
  return aliases
}

export function createCompletionProvider(
  getCatalog: CatalogGetter,
): languages.CompletionItemProvider {
  return {
    triggerCharacters: ['.', '"'],
    provideCompletionItems(model, position) {
      const range = currentWordRange(model, position)
      const prefix = textBefore(model, position)
      const snippet = monacoLanguages.CompletionItemInsertTextRule.InsertAsSnippet
      const suggestions = suggestSql({
        prefix,
        sql: model.getValue(),
        catalog: getCatalog(),
      }).map((item) => ({
        label: {
          label: item.label,
          description: item.detail,
        },
        kind: KIND[item.kind],
        insertText: item.kind === 'table' || item.kind === 'column' ? quoteIdent(item.insertText) : item.insertText,
        insertTextRules: item.snippet ? snippet : undefined,
        documentation: item.documentation ? { value: item.documentation } : undefined,
        range,
        sortText: item.sortText,
      }))
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
      const aliases = aliasesFor(model)
      const qualifier = token.qualifier
        ? aliases.get(token.qualifier.toLowerCase()) ?? token.qualifier
        : undefined
      const table = findTable(catalog, token.name)
      if (table && !qualifier) {
        const line = model.getLineContent(position.lineNumber)
        const after = line.slice(token.range.endColumn - 1)
        if (!shouldHoverTable(after)) return null
        return { range: token.range, contents: [{ value: tableHoverMarkdown(table) }] }
      }
      const column = findColumn(catalog, token.name, qualifier)
      if (column) {
        const parent = findTable(catalog, column.table)
        return { range: token.range, contents: [{ value: columnHoverMarkdown(column, parent) }] }
      }
      const fn = DUCKDB_FUNCTION_BY_NAME.get(token.name.toLowerCase())
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
      const fn = DUCKDB_FUNCTION_BY_NAME.get(match[1].toLowerCase())
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
