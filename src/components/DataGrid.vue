<script setup lang="ts">
import { computed, shallowRef, useTemplateRef } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  FlexRender,
  createColumnHelper,
  getCoreRowModel,
  useVueTable,
} from '@tanstack/vue-table'
import { useVirtualizer } from '@tanstack/vue-virtual'
import { CaretDownOutlined, CaretUpOutlined, FilterOutlined } from '@antdv-next/icons'
import { Empty, Flex, Popover, theme } from 'antdv-next'
import { storeToRefs } from 'pinia'
import ColumnFilter from './ColumnFilter.vue'
import ColumnHeaderTitle from './columns/ColumnHeaderTitle.vue'
import ColumnResizeHandle from './ColumnResizeHandle.vue'
import { useWorkspace } from '../stores/workspace'
import type { ColumnInfo, DistinctValue, FilterSpec, ValueLabel } from '../types'
import { findCondition, hasColumnFilter, removeColumnConditions, upsertCondition } from '../utils/queryRules'
import {
  DEFAULT_COLUMN_WIDTH,
  MAX_COLUMN_WIDTH,
  MIN_COLUMN_WIDTH,
  columnWidthOf,
  pinStickyStyle,
  visiblePinned,
} from '../utils/columnLayout'
import { GRID_ROW_HEIGHT, virtualRowPads } from '../utils/virtualTable'

const { t } = useI18n()
const { token } = theme.useToken()
const store = useWorkspace()
const { page, metadata, metadataByTable, labelMode, headerMode, sorts, offset, hidden, pinnedStart, pinnedEnd, columnWidths, filters, dataTab } = storeToRefs(store)

const scroller = useTemplateRef<HTMLElement>('scroller')
const filterCol = shallowRef<string | null>(null)
const resizingId = shallowRef<string | null>(null)
const skipSort = shallowRef(false)

const labels = computed(() => {
  const map = new Map<string, string>()
  const sources = metadata.value
    ? [metadata.value]
    : Object.values(metadataByTable.value)
  for (const meta of sources) {
    for (const item of meta.valueLabels as ValueLabel[]) {
      if (item.numValue != null) map.set(`${item.labelSet}#n#${item.numValue}`, item.label)
      if (item.strValue != null) map.set(`${item.labelSet}#s#${item.strValue}`, item.label)
    }
  }
  return map
})

const labelSetByName = computed(() => {
  const map = new Map<string, string>()
  for (const col of page.value?.columns ?? []) {
    if (col.labelSet) map.set(col.name, col.labelSet)
  }
  for (const item of metadata.value?.variables ?? []) {
    if (item.labelSet && !map.has(item.name)) map.set(item.name, item.labelSet)
  }
  return map
})

const colByName = computed(() => {
  const map = new Map<string, ColumnInfo>()
  for (const col of page.value?.columns ?? []) map.set(col.name, col)
  return map
})

function lookupValueLabel(labelSet: string, raw: string | number) {
  if (typeof raw === 'number' && Number.isFinite(raw)) {
    return labels.value.get(`${labelSet}#n#${raw}`)
      ?? labels.value.get(`${labelSet}#s#${String(raw)}`)
      ?? null
  }
  const text = String(raw)
  const asNum = Number(text)
  if (text.trim() !== '' && Number.isFinite(asNum)) {
    return labels.value.get(`${labelSet}#n#${asNum}`)
      ?? labels.value.get(`${labelSet}#s#${text}`)
      ?? null
  }
  return labels.value.get(`${labelSet}#s#${text}`) ?? null
}

function formatCell(col: ColumnInfo, raw: string | number | null, mode = labelMode.value) {
  if (raw == null) return ''
  if (mode === 'value') return String(raw)
  const labelSet = col.labelSet ?? labelSetByName.value.get(col.name)
  if (!labelSet) return String(raw)
  const found = lookupValueLabel(labelSet, raw)
  if (!found) return String(raw)
  return mode === 'label' ? found : `${raw} | ${found}`
}

const indexByName = computed(() => {
  const map = new Map<string, number>()
  for (const [index, col] of (page.value?.columns ?? []).entries()) {
    map.set(col.name, index)
  }
  return map
})

const displayCols = computed(() => {
  const cols = page.value?.columns ?? []
  const byName = new Map(cols.map((col) => [col.name, col]))
  const names = store.orderedColumnNames().filter((name) => byName.has(name) && !hidden.value.includes(name))
  return names.map((name) => byName.get(name)!).filter(Boolean)
})

const displayedNames = computed(() => displayCols.value.map((col) => col.name))

const visiblePinnedStart = computed(() =>
  visiblePinned(pinnedStart.value, displayedNames.value),
)

const visiblePinnedEnd = computed(() =>
  visiblePinned(pinnedEnd.value, displayedNames.value),
)

function sizeOf(name: string) {
  return columnWidthOf(name, columnWidths.value)
}

const helper = createColumnHelper<Array<string | number | null>>()
const columnDefs = computed(() => {
  const mode = labelMode.value
  return [
    helper.display({
      id: '_row',
      header: '#',
      size: sizeOf('_row'),
      minSize: 48,
      maxSize: 160,
      cell: (ctx) => String(offset.value + ctx.row.index + 1),
    }),
    ...displayCols.value.map((col) =>
      helper.accessor((row) => {
        const index = indexByName.value.get(col.name)
        return index == null ? null : row[index]
      }, {
        id: col.name,
        header: col.name,
        size: sizeOf(col.name),
        minSize: MIN_COLUMN_WIDTH,
        maxSize: MAX_COLUMN_WIDTH,
        cell: (ctx) => formatCell(col, ctx.getValue(), mode),
      }),
    ),
  ]
})

const tableWidth = computed(() =>
  ['_row', ...displayCols.value.map((col) => col.name)].reduce((sum, name) => sum + sizeOf(name), 0),
)

function pinClass(name: string) {
  if (name === '_row') return 'row-head'
  if (visiblePinnedStart.value.includes(name)) {
    return name === visiblePinnedStart.value.at(-1) ? 'pin-start pin-start-last' : 'pin-start'
  }
  if (visiblePinnedEnd.value.includes(name)) {
    return name === visiblePinnedEnd.value[0] ? 'pin-end pin-end-first' : 'pin-end'
  }
  return ''
}

function pinStyle(name: string) {
  return pinStickyStyle(name, visiblePinnedStart.value, visiblePinnedEnd.value, sizeOf)
}

const table = useVueTable({
  get data() {
    return page.value?.rows ?? []
  },
  get columns() {
    return columnDefs.value
  },
  state: {
    get columnSizing() {
      return columnWidths.value
    },
  },
  onColumnSizingChange: (updater) => {
    store.setColumnWidths(typeof updater === 'function' ? updater(columnWidths.value) : updater)
  },
  defaultColumn: {
    minSize: MIN_COLUMN_WIDTH,
    maxSize: MAX_COLUMN_WIDTH,
    size: DEFAULT_COLUMN_WIDTH,
  },
  columnResizeMode: 'onChange',
  enableColumnResizing: true,
  getCoreRowModel: getCoreRowModel(),
})

const virtualizer = useVirtualizer(
  computed(() => ({
    count: table.getRowModel().rows.length,
    getScrollElement: () => scroller.value,
    estimateSize: () => GRID_ROW_HEIGHT,
    overscan: 12,
  })),
)

const virtualPads = computed(() =>
  virtualRowPads(virtualizer.value.getVirtualItems(), virtualizer.value.getTotalSize()),
)

const sortByName = computed(() => {
  const map = new Map<string, { index: number; desc: boolean; total: number }>()
  const total = sorts.value.length
  for (const [index, item] of sorts.value.entries()) {
    map.set(item.column, { index, desc: item.desc, total })
  }
  return map
})

async function onHeader(name: string, event: MouseEvent) {
  if (!dataTab.value || name === '_row' || skipSort.value || resizingId.value) return
  await store.applySort(name, event.shiftKey)
}

function onResizeStart(header: { id: string; getResizeHandler: () => (event: unknown) => void }, event: MouseEvent | TouchEvent) {
  resizingId.value = header.id
  header.getResizeHandler()(event)
  const stop = () => {
    resizingId.value = null
    skipSort.value = true
    window.setTimeout(() => {
      skipSort.value = false
    }, 0)
    window.removeEventListener('mouseup', stop)
    window.removeEventListener('touchend', stop)
  }
  window.addEventListener('mouseup', stop)
  window.addEventListener('touchend', stop)
}

function resetColumnWidth(name: string) {
  const next = { ...columnWidths.value }
  delete next[name]
  store.setColumnWidths(next)
}

function formatFilterValue(name: string, raw: string) {
  const col = colByName.value.get(name)
  if (!col) return raw
  const asNum = Number(raw)
  const value = col.storageType !== 'string' && raw.trim() !== '' && Number.isFinite(asNum) ? asNum : raw
  return formatCell(col, value) || raw
}

function pageValuesOf(name: string): DistinctValue[] {
  const index = indexByName.value.get(name)
  if (index == null) return []
  const counts = new Map<string | null, number>()
  for (const row of page.value?.rows ?? []) {
    const raw = row[index]
    const key = raw == null || raw === '' ? null : String(raw)
    counts.set(key, (counts.get(key) ?? 0) + 1)
  }
  return [...counts.entries()].map(([value, count]) => ({ value, label: value, count }))
}

function hasFilter(name: string) {
  return hasColumnFilter(filters.value, name)
}

async function applyFilter(spec: FilterSpec) {
  filterCol.value = null
  await store.applyFilters(upsertCondition(store.filters, spec))
}

async function clearFilter(column: string) {
  filterCol.value = null
  await store.applyFilters(removeColumnConditions(store.filters, column))
}

function onFilterOpen(column: string, open: boolean) {
  filterCol.value = open ? column : null
}
</script>

<template>
  <div class="grid-host">
    <Flex v-if="!page" class="grid-empty" align="center" justify="center">
      <Empty :description="t('grid.empty')" />
    </Flex>
    <div
      v-else
      ref="scroller"
      class="grid-scroll"
      :class="{ resizing: Boolean(resizingId), stacked: headerMode === 'both' }"
    >
      <table class="grid-table" :style="{ width: `${tableWidth}px` }">
        <colgroup>
          <col
            v-for="header in table.getHeaderGroups()[0]?.headers ?? []"
            :key="header.id"
            :style="{ width: `${sizeOf(header.id)}px` }"
          >
        </colgroup>
        <thead>
          <tr v-for="group in table.getHeaderGroups()" :key="group.id">
            <th
              v-for="header in group.headers"
              :key="header.id"
                :class="[
                pinClass(header.id),
                {
                  num: header.id !== '_row' && colByName.get(header.id)?.storageType !== 'string',
                  resizing: resizingId === header.id,
                  sortable: Boolean(dataTab) && header.id !== '_row',
                },
              ]"
              :style="pinStyle(header.id)"
              @click="onHeader(header.id, $event)"
            >
              <Flex class="th-inner" align="center" :gap="4" :justify="header.id === '_row' ? 'flex-end' : 'space-between'">
                <span v-if="header.id === '_row'" class="th-label">#</span>
                <span v-else class="th-label">
                  <ColumnHeaderTitle
                    :name="header.id"
                    :label="colByName.get(header.id)?.label ?? null"
                    :mode="headerMode"
                    :storage-type="colByName.get(header.id)?.storageType ?? 'string'"
                    :display-format="colByName.get(header.id)?.displayFormat ?? null"
                    :is-datetime="colByName.get(header.id)?.isDatetime ?? false"
                  />
                </span>
                <span v-if="header.id !== '_row'" class="th-actions" @click.stop>
                  <span v-if="sortByName.get(header.id)" class="sort-mark">
                    <CaretUpOutlined v-if="!sortByName.get(header.id)?.desc" />
                    <CaretDownOutlined v-else />
                    <span v-if="(sortByName.get(header.id)?.total ?? 0) > 1" class="sort-ord">{{ (sortByName.get(header.id)?.index ?? 0) + 1 }}</span>
                  </span>
                  <Popover
                    v-if="dataTab"
                    trigger="click"
                    :open="filterCol === header.id"
                    @open-change="onFilterOpen(header.id, $event)"
                  >
                    <span class="filter-trigger" :class="{ active: hasFilter(header.id) }">
                      <FilterOutlined class="filter-icon" />
                    </span>
                    <template #content>
                      <ColumnFilter
                        :column="header.id"
                        :storage-type="colByName.get(header.id)?.storageType ?? 'string'"
                        :display-format="colByName.get(header.id)?.displayFormat ?? null"
                        :is-datetime="colByName.get(header.id)?.isDatetime ?? false"
                        :existing="findCondition(filters, header.id)"
                        :table="dataTab?.tableName"
                        :page-values="pageValuesOf(header.id)"
                        :format-value="(raw) => formatFilterValue(header.id, raw)"
                        :active="filterCol === header.id"
                        @apply="applyFilter"
                        @clear="clearFilter(header.id)"
                        @cancel="filterCol = null"
                      />
                    </template>
                  </Popover>
                </span>
              </Flex>
              <ColumnResizeHandle
                :active="resizingId === header.id"
                @start="onResizeStart(header, $event)"
                @reset="resetColumnWidth(header.id)"
              />
            </th>
          </tr>
        </thead>
        <tbody>
          <tr v-if="table.getRowModel().rows.length === 0">
            <td :colspan="columnDefs.length">
              <Empty :description="t('grid.noRows')" />
            </td>
          </tr>
          <template v-else>
            <tr v-if="virtualPads.top > 0" class="virtual-pad" aria-hidden="true">
              <td :colspan="columnDefs.length">
                <div :style="{ height: `${virtualPads.top}px` }" />
              </td>
            </tr>
            <tr
              v-for="vrow in virtualizer.getVirtualItems()"
              :key="vrow.index"
              :class="{ even: vrow.index % 2 === 1 }"
              :style="{ height: `${vrow.size}px` }"
            >
              <td
                v-for="cell in table.getRowModel().rows[vrow.index]?.getVisibleCells() ?? []"
                :key="cell.id"
                :class="[
                  pinClass(cell.column.id),
                  { num: cell.column.id !== '_row' && colByName.get(cell.column.id)?.storageType !== 'string' },
                ]"
                :style="pinStyle(cell.column.id)"
              >
                <FlexRender
                  :render="cell.column.columnDef.cell"
                  :props="cell.getContext()"
                />
              </td>
            </tr>
            <tr v-if="virtualPads.bottom > 0" class="virtual-pad" aria-hidden="true">
              <td :colspan="columnDefs.length">
                <div :style="{ height: `${virtualPads.bottom}px` }" />
              </td>
            </tr>
          </template>
        </tbody>
      </table>
    </div>
  </div>
</template>

<style scoped>
.grid-host,
.grid-empty,
.grid-scroll {
  width: 100%;
  height: 100%;
  min-width: 0;
}

.grid-host {
  display: flex;
  flex: 1;
  flex-direction: column;
  min-height: 0;
  overflow: hidden;
  background: v-bind('token.colorBgContainer');
}

.grid-scroll {
  flex: 1;
  min-height: 0;
  overflow: auto;
  overflow-anchor: none;
}

.grid-scroll.resizing {
  cursor: col-resize;
  user-select: none;
}

.grid-table {
  table-layout: fixed;
  border-collapse: separate;
  border-spacing: 0;
  color: v-bind('token.colorText');
  font-variant-numeric: tabular-nums;
}

.grid-table th,
.grid-table td {
  height: 32px;
  padding: 0 10px;
  overflow: hidden;
  white-space: nowrap;
  text-overflow: ellipsis;
  border-right: 1px solid v-bind('token.colorBorderSecondary');
  border-bottom: 1px solid v-bind('token.colorBorderSecondary');
  background-color: v-bind('token.colorBgContainer');
}

.grid-table tr.virtual-pad td {
  height: auto;
  padding: 0;
  overflow: hidden;
  line-height: 0;
  pointer-events: none;
  border: 0;
  background: none;
}

.grid-scroll.stacked .grid-table th {
  height: 44px;
}

.grid-table th {
  position: sticky;
  top: 0;
  z-index: 1;
  font-weight: 600;
  text-align: left;
  user-select: none;
  background-image: linear-gradient(v-bind('token.colorFillAlter'), v-bind('token.colorFillAlter'));
}

.grid-table th.sortable {
  cursor: pointer;
}

.grid-table th.resizing {
  cursor: col-resize;
}

.grid-table td.num,
.grid-table th.num {
  text-align: right;
}

.grid-table tr.even td {
  background-image: linear-gradient(v-bind('token.colorFillQuaternary'), v-bind('token.colorFillQuaternary'));
}

.grid-table tr:hover td {
  background-image: linear-gradient(v-bind('token.controlItemBgHover'), v-bind('token.controlItemBgHover'));
}

.grid-table td.row-head,
.grid-table th.row-head {
  position: sticky;
  left: 0;
  min-width: 56px;
  color: v-bind('token.colorTextSecondary');
  text-align: right;
  background-image: linear-gradient(v-bind('token.colorFillAlter'), v-bind('token.colorFillAlter'));
}

.grid-table td.row-head {
  z-index: 5;
}

.grid-table th.row-head {
  z-index: 6;
}

.grid-table tr.even td.row-head {
  background-image: linear-gradient(v-bind('token.colorFillQuaternary'), v-bind('token.colorFillQuaternary'));
}

.grid-table tr:hover td.row-head {
  background-image: linear-gradient(v-bind('token.controlItemBgHover'), v-bind('token.controlItemBgHover'));
}

.pin-start,
.pin-end {
  position: sticky;
  z-index: 3;
}

.pin-start-last {
  box-shadow: 6px 0 8px -6px rgb(0 0 0 / 18%);
}

.pin-end-first {
  box-shadow: -6px 0 8px -6px rgb(0 0 0 / 18%);
}

.grid-table th.pin-start,
.grid-table th.pin-end {
  z-index: 4;
}

.grid-table tr.even td.pin-start,
.grid-table tr.even td.pin-end {
  background-image: linear-gradient(v-bind('token.colorFillQuaternary'), v-bind('token.colorFillQuaternary'));
}

.grid-table tr:hover td.pin-start,
.grid-table tr:hover td.pin-end {
  background-image: linear-gradient(v-bind('token.controlItemBgHover'), v-bind('token.controlItemBgHover'));
}

.th-inner {
  width: 100%;
  min-width: 0;
}

.th-label {
  flex: 1;
  min-width: 0;
  overflow: hidden;
  line-height: 1.2;
}

.grid-table th :deep(.header-title),
.grid-table th :deep(.ant-tooltip-disabled-compatible-wrapper) {
  min-width: 0;
  max-width: 100%;
}

.th-actions {
  display: inline-flex;
  gap: 6px;
  align-items: center;
  color: v-bind('token.colorTextTertiary');
}

.sort-mark {
  display: inline-flex;
  gap: 2px;
  align-items: center;
}

.sort-ord {
  font-size: 10px;
  font-variant-numeric: tabular-nums;
  line-height: 1;
}

.filter-trigger {
  display: inline-flex;
  color: inherit;
  cursor: pointer;
}

.filter-trigger.active,
.filter-trigger:hover {
  color: v-bind('token.colorPrimary');
}
</style>
