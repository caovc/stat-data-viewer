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
      'editor.foreground': '#0f172a',
      'editor.lineHighlightBackground': '#f4f6f8',
      'editorLineNumber.foreground': '#94a3b8',
      'editorLineNumber.activeForeground': '#0f766e',
      'editorCursor.foreground': '#0f766e',
      'editor.selectionBackground': '#0f766e33',
      'editorWidget.background': '#ffffff',
      'editorWidget.foreground': '#0f172a',
      'editorWidget.border': '#d0d5dd',
      'editorSuggestWidget.background': '#ffffff',
      'editorSuggestWidget.foreground': '#0f172a',
      'editorSuggestWidget.border': '#d0d5dd',
      'editorSuggestWidget.selectedBackground': '#ccfbf1',
      'editorSuggestWidget.selectedForeground': '#134e4a',
      'editorSuggestWidget.highlightForeground': '#0f766e',
      'editorSuggestWidget.focusHighlightForeground': '#0f766e',
      'editorHoverWidget.background': '#ffffff',
      'editorHoverWidget.foreground': '#0f172a',
      'editorHoverWidget.border': '#d0d5dd',
      'editorHoverWidget.statusBarBackground': '#f4f6f8',
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
      'editor.foreground': '#e2e8f0',
      'editor.lineHighlightBackground': '#1d232b',
      'editorLineNumber.foreground': '#64748b',
      'editorLineNumber.activeForeground': '#5eead4',
      'editorCursor.foreground': '#5eead4',
      'editor.selectionBackground': '#0f766e55',
      'editorWidget.background': '#1d232b',
      'editorWidget.foreground': '#e2e8f0',
      'editorWidget.border': '#3f4a57',
      'editorSuggestWidget.background': '#1d232b',
      'editorSuggestWidget.foreground': '#e2e8f0',
      'editorSuggestWidget.border': '#3f4a57',
      'editorSuggestWidget.selectedBackground': '#115e59',
      'editorSuggestWidget.selectedForeground': '#f0fdfa',
      'editorSuggestWidget.highlightForeground': '#5eead4',
      'editorSuggestWidget.focusHighlightForeground': '#5eead4',
      'editorHoverWidget.background': '#1d232b',
      'editorHoverWidget.foreground': '#e2e8f0',
      'editorHoverWidget.border': '#3f4a57',
      'editorHoverWidget.statusBarBackground': '#171c22',
    },
  })
  languageReady = true
}

export function preloadSqlEditor(): void {
  ensureDuckdbLanguage()
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
