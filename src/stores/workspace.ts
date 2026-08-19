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
  ScrollMode,
  SortSpec,
  SqlTab,
  TabView,
  WorkspaceTab,
} from '../types'
import { loadPreferences, savePreferences } from '../preferences'
import { appendPageRows } from '../utils/infiniteScroll'
import { shouldDropInactiveCache, shouldFetchOnActivate } from '../utils/tabCache'
import {
  displayColumnNames,
  mergeColumnOrder,
  moveItem,
  nextPinList,
} from '../utils/columnLayout'
import { emptyFilterGroup, pruneFilterGroup, pruneFiltersToColumns } from '../utils/queryRules'

let unlisten: UnlistenFn | null = null
let idSeq = 1

const DEFAULT_PAGE_SIZE = 300

export function createTabView(): TabView {
  return {
    sorts: [],
    filters: emptyFilterGroup(),
    hidden: [],
    columnOrder: [],
    pinnedStart: [],
    pinnedEnd: [],
    columnWidths: {},
    offset: 0,
    pageSize: DEFAULT_PAGE_SIZE,
    scrollMode: loadPreferences().scrollMode,
  }
}

export const useWorkspace = defineStore('workspace', () => {
  const tabs = ref<WorkspaceTab[]>([])
  const activeId = ref<string | null>(null)
  const labelMode = ref<LabelMode>('value')
  const headerMode = ref<HeaderMode>('name')
  const metadataByTable = shallowRef<Record<string, DatasetMeta>>({})
  const pageById = shallowRef<Record<string, PageResult>>({})
  const loading = ref(false)
  const loadingMore = ref(false)
  const error = ref<string | null>(null)
  let loadMoreGen = 0
  const fallbackSql = ref('SELECT * FROM ')
  const showSql = ref(false)
  const showReimport = ref(false)
  const showExport = ref(false)
  const showColumns = ref(false)
  const showQuery = ref(false)
  const showVariables = ref(false)
  const dragging = ref(false)

  const active = computed(() => tabs.value.find((t) => t.id === activeId.value) ?? null)
  const dataTab = computed(() => (active.value?.kind === 'data' ? active.value : null))
  const sqlCatalog = computed(() => buildSqlCatalog(tabs.value, metadataByTable.value))
  const page = computed(() => (activeId.value ? pageById.value[activeId.value] ?? null : null))
  const metadata = computed(() => {
    const tab = dataTab.value
    return tab ? metadataByTable.value[tab.tableName] ?? null : null
  })

  const sorts = computed({
    get: () => active.value?.view.sorts ?? [],
    set: (next) => {
      if (active.value) active.value.view.sorts = next
    },
  })
  const filters = computed({
    get: () => active.value?.view.filters ?? emptyFilterGroup(),
    set: (next) => {
      if (active.value) active.value.view.filters = next
    },
  })
  const hidden = computed({
    get: () => active.value?.view.hidden ?? [],
    set: (next) => {
      if (active.value) active.value.view.hidden = next
    },
  })
  const columnOrder = computed({
    get: () => active.value?.view.columnOrder ?? [],
    set: (next) => {
      if (active.value) active.value.view.columnOrder = next
    },
  })
  const pinnedStart = computed({
    get: () => active.value?.view.pinnedStart ?? [],
    set: (next) => {
      if (active.value) active.value.view.pinnedStart = next
    },
  })
  const pinnedEnd = computed({
    get: () => active.value?.view.pinnedEnd ?? [],
    set: (next) => {
      if (active.value) active.value.view.pinnedEnd = next
    },
  })
  const columnWidths = computed({
    get: () => active.value?.view.columnWidths ?? {},
    set: (next) => {
      if (active.value) active.value.view.columnWidths = next
    },
  })
  const offset = computed({
    get: () => active.value?.view.offset ?? 0,
    set: (next) => {
      if (active.value) active.value.view.offset = next
    },
  })
  const pageSize = computed({
    get: () => active.value?.view.pageSize ?? DEFAULT_PAGE_SIZE,
    set: (next) => {
      if (active.value) active.value.view.pageSize = next
    },
  })
  const scrollMode = computed({
    get: () => active.value?.view.scrollMode ?? 'page',
    set: (next) => {
      void setScrollMode(next)
    },
  })
  const sqlDraft = computed({
    get: () => (active.value?.kind === 'sql' ? active.value.sql : fallbackSql.value),
    set: (next) => {
      if (active.value?.kind === 'sql') active.value.sql = next
      else fallbackSql.value = next
    },
  })

  function rememberMeta(meta: DatasetMeta) {
    metadataByTable.value = { ...metadataByTable.value, [meta.tableName]: meta }
  }

  function setPage(id: string, next: PageResult | null) {
    const pages = { ...pageById.value }
    if (next) pages[id] = next
    else delete pages[id]
    pageById.value = pages
  }

  function addTab(tab: WorkspaceTab) {
    tabs.value.push(tab)
    activeId.value = tab.id
  }

  async function activate(id: string) {
    if (activeId.value !== id) {
      showReimport.value = false
      showExport.value = false
    }
    activeId.value = id
    const tab = tabs.value.find((t) => t.id === id)
    if (tab?.kind !== 'data') {
      showQuery.value = false
      showVariables.value = false
      return
    }
    if (!shouldFetchOnActivate(pageById.value[id])) return
    await refresh()
  }

  function closeTab(id: string) {
    const idx = tabs.value.findIndex((t) => t.id === id)
    if (idx < 0) return
    const wasActive = activeId.value === id
    tabs.value.splice(idx, 1)
    if (wasActive) {
      showReimport.value = false
      showExport.value = false
      const next = tabs.value[idx] ?? tabs.value[idx - 1] ?? null
      activeId.value = next?.id ?? null
      if (!next) {
        showColumns.value = false
        showQuery.value = false
        showVariables.value = false
      } else {
        void activate(next.id)
      }
    }
    setPage(id, null)
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
        view: createTabView(),
      })
    }
    await refresh()
  }

  function invalidateLoadMore() {
    loadMoreGen += 1
    loadingMore.value = false
  }

  async function fetchChunk(tab: WorkspaceTab, offset: number, pageSize = tab.view.pageSize) {
    if (tab.kind === 'data') {
      return api.queryPage({
        table: tab.tableName,
        offset,
        pageSize,
        sorts: tab.view.sorts,
        filters: tab.view.filters,
        hidden: tab.view.hidden,
      })
    }
    return api.runSql({ sql: tab.sql, offset, pageSize })
  }

  async function refresh(opts?: { silent?: boolean }) {
    const tab = active.value
    if (!tab) return
    if (tab.kind === 'data') await refreshData(tab, opts)
    else if (pageById.value[tab.id]) await refreshSql(tab, opts)
  }

  async function refreshData(tab: DataTab, opts?: { silent?: boolean }) {
    const silent = opts?.silent ?? false
    if (!silent) loading.value = true
    error.value = null
    invalidateLoadMore()
    try {
      const meta = await api.getMetadata(tab.tableName)
      rememberMeta(meta)
      syncColumnOrder(meta.variables.map((item) => item.name))
      const current = pageById.value[tab.id]
      if (
        silent
        && tab.view.scrollMode === 'infinite'
        && current
        && current.rows.length > tab.view.pageSize
      ) {
        const probe = await fetchChunk(tab, 0, 1)
        setPage(tab.id, {
          ...current,
          columns: probe.columns,
          totalRows: probe.totalRows,
          offset: 0,
        })
        return
      }
      setPage(tab.id, await fetchChunk(tab, tab.view.offset))
    } catch (e) {
      error.value = String(e)
    } finally {
      if (!silent) loading.value = false
    }
  }

  async function refreshSql(tab: SqlTab, opts?: { silent?: boolean }) {
    const silent = opts?.silent ?? false
    if (!silent) loading.value = true
    error.value = null
    invalidateLoadMore()
    try {
      const current = pageById.value[tab.id]
      if (
        silent
        && tab.view.scrollMode === 'infinite'
        && current
        && current.rows.length > tab.view.pageSize
      ) {
        const probe = await fetchChunk(tab, 0, 1)
        setPage(tab.id, {
          ...current,
          columns: probe.columns,
          totalRows: probe.totalRows,
          offset: 0,
        })
        syncColumnOrder(probe.columns.map((item) => item.name))
        return
      }
      const result = await fetchChunk(tab, tab.view.offset)
      setPage(tab.id, result)
      syncColumnOrder(result.columns.map((item) => item.name))
    } catch (e) {
      error.value = String(e)
    } finally {
      if (!silent) loading.value = false
    }
  }

  async function runActiveSql() {
    const sql = sqlDraft.value
    loading.value = true
    error.value = null
    try {
      const current = active.value
      const tab = current?.kind === 'sql'
        ? current
        : (() => {
            const next: SqlTab = {
              id: `s${idSeq++}`,
              kind: 'sql',
              title: 'SQL result',
              sql,
              view: createTabView(),
            }
            addTab(next)
            return next
          })()
      tab.sql = sql
      tab.view.offset = 0
      invalidateLoadMore()
      const result = await fetchChunk(tab, 0)
      setPage(tab.id, result)
      syncColumnOrder(result.columns.map((item) => item.name))
    } catch (e) {
      error.value = String(e)
    } finally {
      loading.value = false
    }
  }

  function uniqueSorts(next: SortSpec[]) {
    const seen = new Set<string>()
    return next.filter((item) => {
      if (!item.column || seen.has(item.column)) return false
      seen.add(item.column)
      return true
    })
  }

  async function applySort(column: string, append = false) {
    const view = dataTab.value?.view
    if (!view) return
    if (append) {
      const index = view.sorts.findIndex((item) => item.column === column)
      if (index < 0) {
        view.sorts = [...view.sorts, { column, desc: false }]
      } else if (!view.sorts[index].desc) {
        view.sorts = view.sorts.map((item, i) => (i === index ? { column, desc: true } : item))
      } else {
        view.sorts = view.sorts.filter((_, i) => i !== index)
      }
    } else if (view.sorts.length === 1 && view.sorts[0].column === column) {
      view.sorts = view.sorts[0].desc ? [] : [{ column, desc: true }]
    } else {
      view.sorts = [{ column, desc: false }]
    }
    view.offset = 0
    await refresh()
  }

  async function applySorts(next: SortSpec[]) {
    const view = dataTab.value?.view
    if (!view) return
    view.sorts = uniqueSorts(next)
    view.offset = 0
    await refresh()
  }

  async function applyFilters(next: FilterGroup) {
    const view = dataTab.value?.view
    if (!view) return
    view.filters = pruneFilterGroup(next)
    view.offset = 0
    await refresh()
  }

  async function applyQuery(nextSorts: SortSpec[], nextFilters: FilterGroup) {
    const view = dataTab.value?.view
    if (!view) return
    view.sorts = uniqueSorts(nextSorts)
    view.filters = pruneFilterGroup(nextFilters)
    view.offset = 0
    await refresh()
  }

  async function toggleHidden(name: string) {
    const view = active.value?.view
    if (!view) return
    view.hidden = view.hidden.includes(name)
      ? view.hidden.filter((n) => n !== name)
      : [...view.hidden, name]
    if (dataTab.value) await refresh()
  }

  async function setOffset(next: number) {
    const view = active.value?.view
    if (!view) return
    view.offset = Math.max(0, next)
    await refresh()
  }

  async function setPageSize(next: number) {
    const view = active.value?.view
    if (!view) return
    view.pageSize = next
    view.offset = 0
    await refresh()
  }

  async function setScrollMode(next: ScrollMode) {
    const view = active.value?.view
    if (!view || view.scrollMode === next) return
    view.scrollMode = next
    view.offset = 0
    savePreferences({ ...loadPreferences(), scrollMode: next })
    await refresh()
  }

  async function loadMore() {
    const tab = active.value
    if (!tab || tab.view.scrollMode !== 'infinite') return
    const current = pageById.value[tab.id]
    if (!current || loadingMore.value || current.rows.length >= current.totalRows) return
    const expected = current.rows.length
    loadingMore.value = true
    const gen = ++loadMoreGen
    error.value = null
    try {
      const next = await fetchChunk(tab, expected)
      if (gen !== loadMoreGen || activeId.value !== tab.id) return
      const latest = pageById.value[tab.id]
      if (!latest) return
      const appended = appendPageRows(latest, next)
      if (appended !== latest) setPage(tab.id, appended)
    } catch (e) {
      if (gen === loadMoreGen) error.value = String(e)
    } finally {
      if (gen === loadMoreGen) loadingMore.value = false
    }
  }

  async function setHidden(names: string[]) {
    const view = active.value?.view
    if (!view) return
    view.hidden = names
    if (dataTab.value) await refresh()
  }

  function sourceColumnNames() {
    return (metadata.value?.variables ?? page.value?.columns ?? []).map((item) => item.name)
  }

  function syncColumnOrder(names: string[]) {
    const view = active.value?.view
    if (!view) return
    view.columnOrder = mergeColumnOrder(view.columnOrder, names)
    const known = new Set(names)
    view.pinnedStart = view.pinnedStart.filter((name) => known.has(name))
    view.pinnedEnd = view.pinnedEnd.filter((name) => known.has(name))
    view.hidden = view.hidden.filter((name) => known.has(name))
    view.sorts = view.sorts.filter((item) => known.has(item.column))
    view.filters = pruneFiltersToColumns(view.filters, known)
  }

  function resetColumnLayoutState() {
    const view = active.value?.view
    if (!view) return
    view.columnOrder = []
    view.pinnedStart = []
    view.pinnedEnd = []
    view.hidden = []
    view.columnWidths = {}
  }

  function setColumnWidths(next: Record<string, number>) {
    const view = active.value?.view
    if (view) view.columnWidths = next
  }

  function orderedColumnNames() {
    const view = active.value?.view
    const names = mergeColumnOrder(view?.columnOrder ?? [], sourceColumnNames())
    return displayColumnNames(names, view?.pinnedStart ?? [], view?.pinnedEnd ?? [])
  }

  function reorderColumns(fromName: string, toName: string) {
    const view = active.value?.view
    if (!view || fromName === toName) return
    const names = orderedColumnNames()
    const from = names.indexOf(fromName)
    const to = names.indexOf(toName)
    if (from < 0 || to < 0) return
    const next = moveItem(names, from, to)
    const startSet = new Set(view.pinnedStart)
    const endSet = new Set(view.pinnedEnd)
    if (startSet.has(fromName) && startSet.has(toName)) {
      view.pinnedStart = next.filter((name) => startSet.has(name))
      return
    }
    if (endSet.has(fromName) && endSet.has(toName)) {
      view.pinnedEnd = next.filter((name) => endSet.has(name))
      return
    }
    if (!startSet.has(fromName) && !endSet.has(fromName) && !startSet.has(toName) && !endSet.has(toName)) {
      view.columnOrder = next.filter((name) => !startSet.has(name) && !endSet.has(name))
    }
  }

  function pinColumn(name: string, pin: ColumnPin) {
    const view = active.value?.view
    if (!view) return
    const next = nextPinList(name, pin, view.pinnedStart, view.pinnedEnd)
    view.pinnedStart = next.pinnedStart
    view.pinnedEnd = next.pinnedEnd
  }

  async function resetColumnLayout() {
    resetColumnLayoutState()
    if (metadata.value) syncColumnOrder(metadata.value.variables.map((item) => item.name))
    else if (page.value) syncColumnOrder(page.value.columns.map((item) => item.name))
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
        if (activeId.value === tab.id) {
          await refresh({ silent: Boolean(pageById.value[tab.id]) })
        } else if (shouldDropInactiveCache(false, payload.complete)) {
          setPage(tab.id, null)
        }
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
    metadataByTable,
    sqlCatalog,
    page,
    pageById,
    loading,
    loadingMore,
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
    scrollMode,
    sqlDraft,
    showSql,
    showReimport,
    showExport,
    showColumns,
    showQuery,
    showVariables,
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
    setScrollMode,
    loadMore,
    bindEvents,
  }
})

export function newSqlTab(): SqlTab {
  return { id: `s${idSeq++}`, kind: 'sql', title: 'SQL', sql: 'SELECT * FROM ', view: createTabView() }
}
