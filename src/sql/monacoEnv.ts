import EditorWorker from 'monaco-editor/editor/editor.worker.js?worker'

const env = globalThis as typeof globalThis & {
  MonacoEnvironment?: { getWorker(): Worker }
}
env.MonacoEnvironment = {
  getWorker() {
    return new EditorWorker()
  },
}
