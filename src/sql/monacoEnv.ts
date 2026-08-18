import type { Environment } from 'monaco-editor/editor/editor.api.js'
import EditorWorker from 'monaco-editor/editor/editor.worker.js?worker'

const env = globalThis as typeof globalThis & { MonacoEnvironment?: Environment }
env.MonacoEnvironment = {
  getWorker() {
    return new EditorWorker()
  },
}
