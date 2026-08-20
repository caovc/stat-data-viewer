import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import { open } from '@tauri-apps/plugin-dialog'
import { storeToRefs } from 'pinia'
import { DATA_FILTERS, takePendingOpenPaths } from '../api'
import { newSqlTab, useWorkspace } from '../stores/workspace'
import { consumePendingOpenPaths } from '../utils/associatedFiles'
import { preloadSqlEditor } from '../sql/registerDuckdb'

let opening = false

export function useWorkspaceActions() {
  const { t } = useI18n()
  const store = useWorkspace()
  const { page, offset, pageSize } = storeToRefs(store)

  const currentPage = computed(() =>
    page.value ? Math.floor(offset.value / pageSize.value) + 1 : 1,
  )

  async function openFiles() {
    if (opening) return
    opening = true
    try {
      const selected = await open({
        multiple: true,
        filters: DATA_FILTERS.map(({ key, extensions }) => ({
          name: t(key),
          extensions,
        })),
      })
      const paths = Array.isArray(selected) ? selected : selected ? [selected] : []
      for (const path of paths) await store.openPath(path)
    } finally {
      opening = false
    }
  }

  async function openAssociatedPaths() {
    await consumePendingOpenPaths(takePendingOpenPaths, (path) => store.openPath(path))
  }

  async function bindAssociatedFiles(): Promise<UnlistenFn | null> {
    try {
      const unlisten = await listen('open-files', () => {
        void openAssociatedPaths()
      })
      await openAssociatedPaths()
      return unlisten
    } catch {
      return null
    }
  }

  function addSql() {
    preloadSqlEditor()
    store.addTab(newSqlTab())
    store.showSql = true
  }

  async function changePage(pageNum: number, size: number) {
    if (size !== pageSize.value) {
      await store.setPageSize(size)
      return
    }
    await store.setOffset((pageNum - 1) * size)
  }

  return {
    store,
    currentPage,
    openFiles,
    bindAssociatedFiles,
    addSql,
    changePage,
  }
}
