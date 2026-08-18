import { defineStore } from 'pinia'
import { computed, ref, shallowRef } from 'vue'
import { i18n } from '../i18n'
import { listen, type UnlistenFn } from '@tauri-apps/api/event'
import * as api from '../api'
import { buildSqlCatalog } from '../sql/catalog'
import type {
  ColumnPin,
  DataTab,
  DatasetMeta,
  FilterGroup,
  ImportEvent,
  HeaderMode,
  LabelMode,
  PageResult,
  SortSpec,
  SqlTab,
  WorkspaceTab,
} from '../types'
import {
  displayColumnNames,
  mergeColumnOrder,
  moveItem,
  nextPinList,
} from '../utils/columnLayout'
import { emptyFilterGroup, pruneFilterGroup } from '../utils/queryRules'

let unlisten: UnlistenFn | null = null
let idSeq = 1

export const useWorkspace = defineStore('workspace', () => {
  const tabs = ref<WorkspaceTab[]>([])
  const activeId = ref<string | null>(null)
  const labelMode = ref<LabelMode>('value')
  const headerMode = ref<HeaderMode>('name')
  const metadata = shallowRef<DatasetMeta | null>(null)
  const metadataByTable = shallowRef<Record<string, DatasetMeta>>({})
  const page = shallowRef<PageResult | null>(null)
  const loading = ref(false)
  const error = ref<string | null>(null)
  const sorts = ref<SortSpec[]>([])
  const filters = ref<FilterGroup>(emptyFilterGroup())
  const hidden = ref<string[]>([])
  const columnOrder = ref<string[]>([])
  const pinnedStart = ref<string[]>([])
  const pinnedEnd = ref<string[]>([])
  const columnWidths = ref<Record<string, number>>({})
  const offset = ref(0)
  const pageSize = ref(300)
  const sqlDraft = ref('SELECT * FROM ')
  const showSql = ref(false)
  const showReimport = ref(false)
  const showExport = ref(false)
  const showColumns = ref(false)
  const showQuery = ref(false)
  const dragging = ref(false)

  const active = computed(() => tabs.value.find((t) => t.id === activeId.value) ?? null)
  const dataTab = computed(() => (active.value?.kind === 'data' ? active.value : null))
  const sqlCatalog = computed(() => buildSqlCatalog(tabs.value, metadataByTable.value))

  function rememberMeta(meta: DatasetMeta) {
    metadataByTable.value = { ...metadataByTable.value, [meta.tableName]: meta }
  }

  function addTab(tab: WorkspaceTab) {
    tabs.value.push(tab)
    activeId.value = tab.id
  }

  async function activate(id: string) {
    activeId.value = id
    const tab = tabs.value.find((t) => t.id === id)
    if (tab?.kind === 'data') {
      offset.value = 0
      sorts.value = []
      filters.value = emptyFilterGroup()
      resetColumnLayoutState()
      await refresh()
    } else if (tab?.kind === 'sql') {
      sqlDraft.value = tab.sql
    }
  }

  function closeTab(id: string) {
    const idx = tabs.value.findIndex((t) => t.id === id)
    if (idx < 0) return
    tabs.value.splice(idx, 1)
    if (activeId.value === id) {
      const next = tabs.value[idx] ?? tabs.value[idx - 1] ?? null
      activeId.value = next?.id ?? null
      if (next?.kind === 'data') void refresh()
    }
  }

  async function openPath(path: string, extra?: { encoding?: string; format?: string; catalogPath?: string }) {
    if (path.toLowerCase().endsWith('.sas7bcat')) {
      error.value = i18n.global.t('errors.catalogOnly')
      return
    }
    error.value = null
    const result = await api.openDataset({ path, ...extra })
    const title = path.split(/[\\/]/).pop() ?? result.tableName
    const existing = tabs.value.find((t) => t.kind === 'data' && t.tableName === result.tableName)
    if (existing) {
      activeId.value = existing.id
    } else {
      addTab({
        id: `t${idSeq++}`,
        kind: 'data',
        title,
        tableName: result.tableName,
        path,
        jobId: result.jobId,
        importing: !result.reused && !result.importComplete,
        progress: result.reused ? 1 : 0,
        error: null,
      })
    }
    await refresh()
  }

  async function refresh() {
    const tab = dataTab.value
    if (!tab) {
      metadata.value = null
      page.value = null
      return
    }
    loading.value = true
    error.value = null
    try {
      metadata.value = await api.getMetadata(tab.tableName)
      rememberMeta(metadata.value)
      syncColumnOrder(metadata.value.variables.map((item) => item.name))
      page.value = await api.queryPage({
        table: tab.tableName,
        offset: offset.value,
        pageSize: pageSize.value,
        sorts: sorts.value,
        filters: filters.value,
        hidden: hidden.value,
      })
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  async function runActiveSql() {
    const sql = sqlDraft.value
    loading.value = true
    error.value = null
    try {
      const result = await api.runSql({ sql, offset: offset.value, pageSize: pageSize.value })
      page.value = result
      const current = active.value
      if (current?.kind === 'sql') current.sql = sql
      else {
        addTab({
          id: `s${idSeq++}`,
          kind: 'sql',
          title: 'SQL result',
          sql,
        })
      }
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  async function applySort(column: string, append = false) {
    if (!dataTab.value) return
    if (append) {
      const index = sorts.value.findIndex((item) => item.column === column)
      if (index < 0) {
        sorts.value = [...sorts.value, { column, desc: false }]
      } else if (!sorts.value[index].desc) {
        sorts.value = sorts.value.map((item, i) => (i === index ? { column, desc: true } : item))
      } else {
        sorts.value = sorts.value.filter((_, i) => i !== index)
      }
    } else if (sorts.value.length === 1 && sorts.value[0].column === column) {
      sorts.value = sorts.value[0].desc ? [] : [{ column, desc: true }]
    } else {
      sorts.value = [{ column, desc: false }]
    }
    offset.value = 0
    await refresh()
  }

  async function applySorts(next: SortSpec[]) {
    if (!dataTab.value) return
    const seen = new Set<string>()
    sorts.value = next.filter((item) => {
      if (!item.column || seen.has(item.column)) return false
      seen.add(item.column)
      return true
    })
    offset.value = 0
    await refresh()
  }

  async function applyFilters(next: FilterGroup) {
    if (!dataTab.value) return
    filters.value = pruneFilterGroup(next)
    offset.value = 0
    await refresh()
  }

  async function applyQuery(nextSorts: SortSpec[], nextFilters: FilterGroup) {
    if (!dataTab.value) return
    const seen = new Set<string>()
    sorts.value = nextSorts.filter((item) => {
      if (!item.column || seen.has(item.column)) return false
      seen.add(item.column)
      return true
    })
    filters.value = pruneFilterGroup(nextFilters)
    offset.value = 0
    await refresh()
  }

  async function toggleHidden(name: string) {
    hidden.value = hidden.value.includes(name)
      ? hidden.value.filter((n) => n !== name)
      : [...hidden.value, name]
    await refresh()
  }

  async function setOffset(next: number) {
    offset.value = Math.max(0, next)
    await refresh()
  }

  async function setPageSize(next: number) {
    pageSize.value = next
    offset.value = 0
    await refresh()
  }

  async function setHidden(names: string[]) {
    hidden.value = names
    await refresh()
  }

  function sourceColumnNames() {
    return (metadata.value?.variables ?? page.value?.columns ?? []).map((item) => item.name)
  }

  function syncColumnOrder(names: string[]) {
    columnOrder.value = mergeColumnOrder(columnOrder.value, names)
    const known = new Set(names)
    pinnedStart.value = pinnedStart.value.filter((name) => known.has(name))
    pinnedEnd.value = pinnedEnd.value.filter((name) => known.has(name))
    hidden.value = hidden.value.filter((name) => known.has(name))
  }

  function resetColumnLayoutState() {
    columnOrder.value = []
    pinnedStart.value = []
    pinnedEnd.value = []
    hidden.value = []
    columnWidths.value = {}
  }

  function setColumnWidths(next: Record<string, number>) {
    columnWidths.value = next
  }

  function orderedColumnNames() {
    const names = mergeColumnOrder(columnOrder.value, sourceColumnNames())
    return displayColumnNames(names, pinnedStart.value, pinnedEnd.value)
  }

  function reorderColumns(fromName: string, toName: string) {
    if (fromName === toName) return
    const names = orderedColumnNames()
    const from = names.indexOf(fromName)
    const to = names.indexOf(toName)
    if (from < 0 || to < 0) return
    const next = moveItem(names, from, to)
    const startSet = new Set(pinnedStart.value)
    const endSet = new Set(pinnedEnd.value)
    if (startSet.has(fromName) && startSet.has(toName)) {
      pinnedStart.value = next.filter((name) => startSet.has(name))
      return
    }
    if (endSet.has(fromName) && endSet.has(toName)) {
      pinnedEnd.value = next.filter((name) => endSet.has(name))
      return
    }
    if (!startSet.has(fromName) && !endSet.has(fromName) && !startSet.has(toName) && !endSet.has(toName)) {
      columnOrder.value = next.filter((name) => !startSet.has(name) && !endSet.has(name))
    }
  }

  function pinColumn(name: string, pin: ColumnPin) {
    const next = nextPinList(name, pin, pinnedStart.value, pinnedEnd.value)
    pinnedStart.value = next.pinnedStart
    pinnedEnd.value = next.pinnedEnd
  }

  async function resetColumnLayout() {
    resetColumnLayoutState()
    if (metadata.value) syncColumnOrder(metadata.value.variables.map((item) => item.name))
    await refresh()
  }

  async function hydrateSqlCatalog() {
    const names = tabs.value
      .filter((tab): tab is DataTab => tab.kind === 'data')
      .map((tab) => tab.tableName)
      .filter((name) => !metadataByTable.value[name])
    if (names.length === 0) return
    const entries = await Promise.all(
      names.map(async (name) => {
        try {
          return await api.getMetadata(name)
        } catch {
          return null
        }
      }),
    )
    const next = { ...metadataByTable.value }
    for (const meta of entries) {
      if (meta) next[meta.tableName] = meta
    }
    metadataByTable.value = next
  }

  async function bindEvents() {
    if (unlisten) return
    unlisten = await listen<ImportEvent>('import-progress', async (event) => {
      const payload = event.payload
      const tab = tabs.value.find((t) => t.kind === 'data' && t.tableName === payload.tableName) as DataTab | undefined
      if (!tab) return
      tab.progress = payload.progress
      tab.importing = !payload.complete && !payload.error
      tab.error = payload.error
      if (payload.error) error.value = payload.error
      if (payload.previewReady || payload.complete) {
        if (activeId.value === tab.id) await refresh()
      }
    })
  }

  return {
    tabs,
    activeId,
    active,
    dataTab,
    labelMode,
    headerMode,
    metadata,
    sqlCatalog,
    page,
    loading,
    error,
    sorts,
    filters,
    hidden,
    columnOrder,
    pinnedStart,
    pinnedEnd,
    columnWidths,
    offset,
    pageSize,
    sqlDraft,
    showSql,
    showReimport,
    showExport,
    showColumns,
    showQuery,
    dragging,
    addTab,
    activate,
    closeTab,
    openPath,
    refresh,
    hydrateSqlCatalog,
    runActiveSql,
    applySort,
    applySorts,
    applyFilters,
    applyQuery,
    toggleHidden,
    setHidden,
    orderedColumnNames,
    reorderColumns,
    pinColumn,
    resetColumnLayout,
    setColumnWidths,
    setOffset,
    setPageSize,
    bindEvents,
  }
})

export function newSqlTab(): SqlTab {
  return { id: `s${idSeq++}`, kind: 'sql', title: 'SQL', sql: 'SELECT * FROM ' }
}
