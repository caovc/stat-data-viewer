export type LabelMode = 'value' | 'label' | 'both'
export type HeaderMode = 'name' | 'label' | 'both'
export type TabKind = 'data' | 'sql'
export type ColumnPin = 'start' | 'end' | null
export const SCROLL_MODES = ['page', 'infinite'] as const
export type ScrollMode = (typeof SCROLL_MODES)[number]

export interface ColumnSetting {
  name: string
  label: string | null
  visible: boolean
  pin: ColumnPin
  storageType: string
  displayFormat: string | null
  isDatetime: boolean
}

export interface ColumnInfo {
  name: string
  label: string | null
  storageType: string
  displayFormat: string | null
  origin: string
  isDatetime: boolean
  labelSet: string | null
}

export interface PageResult {
  columns: ColumnInfo[]
  rows: Array<Array<string | number | null>>
  offset: number
  pageSize: number
  totalRows: number
}

export type FilterCombinator = 'and' | 'or'

export interface FilterSpec {
  column: string
  op: string
  value?: string
  value2?: string
  values?: string[]
  includeNull?: boolean
}

export interface FilterCondition extends FilterSpec {
  type: 'condition'
}

export interface FilterGroup {
  type: 'group'
  combinator: FilterCombinator
  children: FilterNode[]
}

export type FilterNode = FilterCondition | FilterGroup

export interface DistinctValue {
  value: string | null
  label: string | null
  count: number
}

export interface DistinctResult {
  values: DistinctValue[]
  truncated: boolean
  emptyCount: number
}

export interface SortSpec {
  column: string
  desc: boolean
}

export interface VariableMeta {
  index: number
  name: string
  label: string | null
  storageType: string
  displayFormat: string | null
  measure: string | null
  displayWidth: number | null
  decimals: number | null
  missingRules: string | null
  labelSet: string | null
}

export interface ValueLabel {
  labelSet: string
  numValue: number | null
  strValue: string | null
  tag: string | null
  label: string
}

export interface DatasetMeta {
  tableName: string
  sourcePath: string | null
  fileFormat: string | null
  encoding: string | null
  fileLabel: string | null
  formatVersion: number | null
  rowCount: number | null
  varCount: number | null
  catalogPath: string | null
  importComplete: boolean
  variables: VariableMeta[]
  valueLabels: ValueLabel[]
}

export interface OpenResult {
  jobId: string
  tableName: string
  reused: boolean
  importComplete: boolean
}

export interface ImportEvent {
  jobId: string
  tableName: string
  progress: number
  rowsImported: number
  previewReady: boolean
  complete: boolean
  error: string | null
}

export interface TabView {
  sorts: SortSpec[]
  filters: FilterGroup
  hidden: string[]
  columnOrder: string[]
  pinnedStart: string[]
  pinnedEnd: string[]
  columnWidths: Record<string, number>
  offset: number
  pageSize: number
  scrollMode: ScrollMode
}

export interface DataTab {
  id: string
  kind: 'data'
  title: string
  tableName: string
  path: string
  jobId: string
  importing: boolean
  progress: number
  error: string | null
  view: TabView
}

export interface SqlTab {
  id: string
  kind: 'sql'
  title: string
  sql: string
  view: TabView
}

export type WorkspaceTab = DataTab | SqlTab
