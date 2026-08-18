import * as monaco from './monaco'
import {
  DUCKDB_LANGUAGE_ID,
  duckdbLanguageConfig,
  duckdbMonarch,
} from './duckdbLanguage'
import {
  createCompletionProvider,
  createHoverProvider,
  createSignatureHelpProvider,
  type CatalogGetter,
} from './sqlIntellisense'

let languageReady = false
let features: Array<{ dispose(): void }> = []

export function ensureDuckdbLanguage(): void {
  if (languageReady) return
  monaco.languages.register({ id: DUCKDB_LANGUAGE_ID, aliases: ['DuckDB SQL', 'duckdb'] })
  monaco.languages.setLanguageConfiguration(DUCKDB_LANGUAGE_ID, duckdbLanguageConfig)
  monaco.languages.setMonarchTokensProvider(DUCKDB_LANGUAGE_ID, duckdbMonarch)
  monaco.editor.defineTheme('stat-data-sql-light', {
    base: 'vs',
    inherit: true,
    rules: [
      { token: 'keyword', foreground: '0f766e', fontStyle: 'bold' },
      { token: 'predefined', foreground: '1d4ed8' },
      { token: 'type', foreground: '7c3aed' },
      { token: 'string', foreground: 'b45309' },
      { token: 'number', foreground: '15803d' },
      { token: 'comment', foreground: '64748b', fontStyle: 'italic' },
      { token: 'operator', foreground: '475569' },
      { token: 'delimiter', foreground: '64748b' },
      { token: 'identifier', foreground: '0f172a' },
    ],
    colors: {
      'editor.background': '#ffffff',
      'editor.lineHighlightBackground': '#f4f6f8',
      'editorLineNumber.foreground': '#94a3b8',
      'editorLineNumber.activeForeground': '#0f766e',
      'editorCursor.foreground': '#0f766e',
      'editor.selectionBackground': '#0f766e33',
      'editorWidget.background': '#ffffff',
      'editorSuggestWidget.background': '#ffffff',
      'editorSuggestWidget.selectedBackground': '#ccfbf1',
      'editorHoverWidget.background': '#ffffff',
    },
  })
  monaco.editor.defineTheme('stat-data-sql-dark', {
    base: 'vs-dark',
    inherit: true,
    rules: [
      { token: 'keyword', foreground: '5eead4', fontStyle: 'bold' },
      { token: 'predefined', foreground: '93c5fd' },
      { token: 'type', foreground: 'c4b5fd' },
      { token: 'string', foreground: 'fbbf24' },
      { token: 'number', foreground: '86efac' },
      { token: 'comment', foreground: '94a3b8', fontStyle: 'italic' },
      { token: 'operator', foreground: 'cbd5e1' },
      { token: 'delimiter', foreground: '94a3b8' },
      { token: 'identifier', foreground: 'e2e8f0' },
    ],
    colors: {
      'editor.background': '#171c22',
      'editor.lineHighlightBackground': '#1d232b',
      'editorLineNumber.foreground': '#64748b',
      'editorLineNumber.activeForeground': '#5eead4',
      'editorCursor.foreground': '#5eead4',
      'editor.selectionBackground': '#0f766e55',
      'editorWidget.background': '#1d232b',
      'editorSuggestWidget.background': '#1d232b',
      'editorSuggestWidget.selectedBackground': '#134e4a',
      'editorHoverWidget.background': '#1d232b',
    },
  })
  languageReady = true
}

export function bindDuckdbIntellisense(getCatalog: CatalogGetter): void {
  for (const item of features) item.dispose()
  features = [
    monaco.languages.registerCompletionItemProvider(
      DUCKDB_LANGUAGE_ID,
      createCompletionProvider(getCatalog),
    ),
    monaco.languages.registerHoverProvider(DUCKDB_LANGUAGE_ID, createHoverProvider(getCatalog)),
    monaco.languages.registerSignatureHelpProvider(
      DUCKDB_LANGUAGE_ID,
      createSignatureHelpProvider(),
    ),
  ]
}
