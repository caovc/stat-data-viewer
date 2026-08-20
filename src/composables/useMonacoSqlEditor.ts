import { onBeforeUnmount, shallowRef, toValue, watch, type MaybeRefOrGetter, type Ref } from 'vue'
import * as monaco from '../sql/monaco'
import { bindDuckdbIntellisense, ensureDuckdbLanguage } from '../sql/registerDuckdb'
import { DUCKDB_LANGUAGE_ID } from '../sql/duckdbLanguage'
import type { SqlCatalog } from '../sql/catalog'

export function useMonacoSqlEditor(options: {
  container: MaybeRefOrGetter<HTMLElement | null>
  value: Ref<string>
  catalog: MaybeRefOrGetter<SqlCatalog>
  theme?: MaybeRefOrGetter<string>
  placeholder?: MaybeRefOrGetter<string>
  visible?: MaybeRefOrGetter<boolean>
  onRun: () => void
}) {
  const editorRef = shallowRef<monaco.editor.IStandaloneCodeEditor | null>(null)
  let applying = false
  let observer: ResizeObserver | null = null
  let layoutTimer = 0
  let createTimer = 0

  ensureDuckdbLanguage()
  bindDuckdbIntellisense(() => toValue(options.catalog))

  const stop = watch(
    () => toValue(options.container),
    (el) => {
      disposeEditor()
      if (!el) return
      createTimer = window.setTimeout(() => {
        if (!el.isConnected) return
        const editor = monaco.editor.create(el, {
          value: options.value.value,
          language: DUCKDB_LANGUAGE_ID,
          theme: toValue(options.theme) ?? 'stat-data-sql-light',
          automaticLayout: false,
          fixedOverflowWidgets: true,
          fontSize: 13,
          lineHeight: 20,
          fontFamily: '"SFMono-Regular", Consolas, Menlo, monospace',
          minimap: { enabled: false },
          scrollBeyondLastLine: false,
          wordWrap: 'on',
          tabSize: 2,
          padding: { top: 8, bottom: 8 },
          overviewRulerLanes: 0,
          hideCursorInOverviewRuler: true,
          renderLineHighlight: 'line',
          glyphMargin: false,
          folding: false,
          lineDecorationsWidth: 8,
          lineNumbersMinChars: 3,
          quickSuggestions: { other: 'on', comments: 'off', strings: 'off' },
          quickSuggestionsDelay: 180,
          suggestOnTriggerCharacters: true,
          acceptSuggestionOnEnter: 'on',
          placeholder: toValue(options.placeholder) ?? '',
          wordBasedSuggestions: 'off',
          snippetSuggestions: 'inline',
          parameterHints: { enabled: true },
          hover: { enabled: 'on', delay: 420, sticky: false },
          suggestFontSize: 12,
          suggestLineHeight: 24,
          suggest: {
            insertMode: 'insert',
            snippetsPreventQuickSuggestions: true,
            localityBonus: true,
            preview: false,
            showInlineDetails: true,
          },
          scrollbar: { verticalScrollbarSize: 8, horizontalScrollbarSize: 8 },
        })
        editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.Enter, options.onRun)
        editor.onDidChangeModelContent(() => {
          if (applying) return
          options.value.value = editor.getValue()
        })
        observer = new ResizeObserver(() => {
          window.clearTimeout(layoutTimer)
          layoutTimer = window.setTimeout(() => editor.layout(), 50)
        })
        observer.observe(el)
        editor.layout()
        editorRef.value = editor
      }, 0)
    },
    { immediate: true, flush: 'post' },
  )

  watch(options.value, (next) => {
    const editor = editorRef.value
    if (!editor || editor.getValue() === next) return
    applying = true
    editor.setValue(next)
    applying = false
  })

  watch(
    () => toValue(options.theme),
    (next) => {
      if (next) monaco.editor.setTheme(next)
    },
  )

  watch(
    () => toValue(options.placeholder),
    (next) => {
      editorRef.value?.updateOptions({ placeholder: next ?? '' })
    },
  )

  watch(
    () => toValue(options.visible),
    (visible) => {
      if (visible === false) return
      window.requestAnimationFrame(() => editorRef.value?.layout())
    },
  )

  onBeforeUnmount(() => {
    stop()
    disposeEditor()
  })

  function disposeEditor() {
    window.clearTimeout(createTimer)
    window.clearTimeout(layoutTimer)
    observer?.disconnect()
    observer = null
    editorRef.value?.dispose()
    editorRef.value = null
  }

  return { editor: editorRef }
}
