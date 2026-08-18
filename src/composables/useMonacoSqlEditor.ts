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
  onRun: () => void
}) {
  const editorRef = shallowRef<monaco.editor.IStandaloneCodeEditor | null>(null)
  let applying = false
  let observer: ResizeObserver | null = null

  ensureDuckdbLanguage()
  bindDuckdbIntellisense(() => toValue(options.catalog))

  const stop = watch(
    () => toValue(options.container),
    (el) => {
      disposeEditor()
      if (!el) return
      const editor = monaco.editor.create(el, {
        value: options.value.value,
        language: DUCKDB_LANGUAGE_ID,
        theme: toValue(options.theme) ?? 'stat-data-sql-light',
        automaticLayout: true,
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
        suggestOnTriggerCharacters: true,
        acceptSuggestionOnEnter: 'on',
        placeholder: toValue(options.placeholder) ?? '',
        wordBasedSuggestions: 'off',
        snippetSuggestions: 'inline',
        parameterHints: { enabled: true },
        hover: { enabled: 'on', delay: 180 },
        scrollbar: { verticalScrollbarSize: 8, horizontalScrollbarSize: 8 },
      })
      editor.addCommand(monaco.KeyMod.CtrlCmd | monaco.KeyCode.Enter, options.onRun)
      editor.onDidChangeModelContent(() => {
        if (applying) return
        options.value.value = editor.getValue()
      })
      observer = new ResizeObserver(() => editor.layout())
      observer.observe(el)
      editorRef.value = editor
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

  onBeforeUnmount(() => {
    stop()
    disposeEditor()
  })

  function disposeEditor() {
    observer?.disconnect()
    observer = null
    editorRef.value?.dispose()
    editorRef.value = null
  }

  return { editor: editorRef }
}
