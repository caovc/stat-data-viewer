import type { FilterSpec } from '../types'
import { columnTypeKind, type ColumnTypeKind, type ColumnTypeInput } from './columnType'

export type { ColumnTypeKind }

export type FilterMode = 'condition' | 'values'

export function defaultFilterOp(kind: ColumnTypeKind) {
  if (kind === 'string') return 'contains'
  return 'eq'
}

export function filterOpsFor(kind: ColumnTypeKind) {
  if (kind === 'string') {
    return ['contains', 'eq', 'ne', 'starts', 'ends', 'is_null', 'not_null'] as const
  }
  if (kind === 'datetime') {
    return ['eq', 'ne', 'gt', 'gte', 'lt', 'lte', 'between', 'is_null', 'not_null'] as const
  }
  return ['eq', 'ne', 'gt', 'gte', 'lt', 'lte', 'between', 'is_null', 'not_null'] as const
}

export function filterOpLabelKey(kind: ColumnTypeKind, op: string) {
  if (kind === 'datetime') {
    if (op === 'gt') return 'filter.after'
    if (op === 'gte') return 'filter.onOrAfter'
    if (op === 'lt') return 'filter.before'
    if (op === 'lte') return 'filter.onOrBefore'
  }
  const keys: Record<string, string> = {
    contains: 'filter.contains',
    eq: 'filter.equals',
    ne: 'filter.notEqual',
    starts: 'filter.starts',
    ends: 'filter.ends',
    gt: 'filter.gt',
    gte: 'filter.gte',
    lt: 'filter.lt',
    lte: 'filter.lte',
    between: 'filter.between',
    is_null: 'filter.isNull',
    not_null: 'filter.notNull',
  }
  return keys[op] ?? op
}

export function filterNeedsValue(op: string) {
  return op !== 'is_null' && op !== 'not_null' && op !== 'empty' && op !== 'not_empty'
}

export function filterNeedsRange(op: string) {
  return op === 'between'
}

export function isActiveFilter(spec: FilterSpec) {
  if (spec.op === 'is_null' || spec.op === 'not_null' || spec.op === 'empty' || spec.op === 'not_empty') {
    return true
  }
  if (spec.op === 'in' || spec.op === 'not_in') {
    return (spec.values?.length ?? 0) > 0 || Boolean(spec.includeNull)
  }
  if (spec.op === 'between') return Boolean(spec.value && spec.value2)
  return Boolean(spec.value)
}

export function filterModeOf(spec?: FilterSpec | null): FilterMode {
  if (!spec) return 'values'
  return spec.op === 'in' || spec.op === 'not_in' ? 'values' : 'condition'
}

export function columnFilterKind(input: ColumnTypeInput): ColumnTypeKind {
  return columnTypeKind(input)
}
