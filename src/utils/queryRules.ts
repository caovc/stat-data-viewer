import type {
  FilterCombinator,
  FilterCondition,
  FilterGroup,
  FilterNode,
  FilterSpec,
  SortSpec,
} from '../types'
import { isActiveFilter } from './columnFilter'

export interface SortDraft {
  id: string
  column: string
  desc: boolean
}

export interface FilterConditionDraft extends FilterCondition {
  id: string
}

export interface FilterGroupDraft {
  type: 'group'
  id: string
  combinator: FilterCombinator
  children: FilterNodeDraft[]
}

export type FilterNodeDraft = FilterConditionDraft | FilterGroupDraft

export interface QueryColumn {
  name: string
  label: string | null
  storageType: string
  displayFormat: string | null
  isDatetime: boolean
}

let ruleSeq = 1

export function nextRuleId(prefix: string) {
  return `${prefix}${ruleSeq++}`
}

export function emptyFilterGroup(): FilterGroup {
  return { type: 'group', combinator: 'and', children: [] }
}

export function emptyFilterDraft(): FilterGroupDraft {
  return { type: 'group', id: nextRuleId('g'), combinator: 'and', children: [] }
}

export function cloneSorts(items: SortSpec[]): SortDraft[] {
  return items.map((item) => ({ id: nextRuleId('s'), column: item.column, desc: item.desc }))
}

export function firstUnusedColumn(columns: QueryColumn[], used: string[]) {
  const taken = new Set(used)
  return columns.find((col) => !taken.has(col.name)) ?? columns[0] ?? null
}

export function toSortSpecs(items: SortDraft[]): SortSpec[] {
  const seen = new Set<string>()
  const next: SortSpec[] = []
  for (const item of items) {
    if (!item.column || seen.has(item.column)) continue
    seen.add(item.column)
    next.push({ column: item.column, desc: item.desc })
  }
  return next
}

export function sameSorts(a: SortSpec[], b: SortSpec[]) {
  return a.length === b.length && a.every((item, i) => item.column === b[i].column && item.desc === b[i].desc)
}

function cloneNode(node: FilterNode): FilterNodeDraft {
  if (node.type === 'group') {
    return {
      type: 'group',
      id: nextRuleId('g'),
      combinator: node.combinator,
      children: node.children.map(cloneNode),
    }
  }
  return {
    ...node,
    id: nextRuleId('f'),
    values: node.values ? [...node.values] : undefined,
  }
}

export function cloneFilterTree(group: FilterGroup | null | undefined): FilterGroupDraft {
  if (!group) return emptyFilterDraft()
  return cloneNode(group) as FilterGroupDraft
}

function toCondition(spec: FilterSpec): FilterCondition {
  return {
    type: 'condition',
    column: spec.column,
    op: spec.op,
    value: spec.value,
    value2: spec.value2,
    values: spec.values ? [...spec.values] : undefined,
    includeNull: spec.includeNull,
  }
}

function stripNode(node: FilterNodeDraft): FilterNode {
  if (node.type === 'group') {
    return {
      type: 'group',
      combinator: node.combinator,
      children: node.children.map(stripNode),
    }
  }
  return toCondition(node)
}

export function pruneFilterGroup(group: FilterGroup): FilterGroup {
  const children: FilterNode[] = []
  for (const child of group.children) {
    if (child.type === 'group') {
      const next = pruneFilterGroup(child)
      if (next.children.length > 0) children.push(next)
    } else if (child.column && isActiveFilter(child)) {
      children.push(child)
    }
  }
  return { type: 'group', combinator: group.combinator, children }
}

export function toFilterGroup(draft: FilterGroupDraft): FilterGroup {
  return pruneFilterGroup(stripNode(draft) as FilterGroup)
}

export function sameFilters(a: FilterGroup, b: FilterGroup) {
  return JSON.stringify(pruneFilterGroup(a)) === JSON.stringify(pruneFilterGroup(b))
}

export function filterCount(group: FilterGroup | null | undefined): number {
  if (!group) return 0
  return countNodes(group)
}

function countNodes(node: FilterNode): number {
  if (node.type === 'condition') return 1
  return node.children.reduce((sum, child) => sum + countNodes(child), 0)
}

export function hasColumnFilter(group: FilterGroup | null | undefined, column: string) {
  return Boolean(group && findCondition(group, column))
}

export function findCondition(node: FilterNode, column: string): FilterCondition | null {
  if (node.type === 'condition') return node.column === column ? node : null
  for (const child of node.children) {
    const found = findCondition(child, column)
    if (found) return found
  }
  return null
}

export function upsertCondition(group: FilterGroup, spec: FilterSpec): FilterGroup {
  let replaced = false
  const next = mapConditions(group, (item) => {
    if (item.column !== spec.column || replaced) return item
    replaced = true
    return isActiveFilter(spec) ? toCondition(spec) : null
  })
  if (!replaced && isActiveFilter(spec)) {
    return { ...next, children: [...next.children, toCondition(spec)] }
  }
  return next
}

export function removeColumnConditions(group: FilterGroup, column: string): FilterGroup {
  return mapConditions(group, (item) => (item.column === column ? null : item))
}

function mapConditions(group: FilterGroup, fn: (item: FilterCondition) => FilterCondition | null): FilterGroup {
  const children: FilterNode[] = []
  for (const child of group.children) {
    if (child.type === 'condition') {
      const next = fn(child)
      if (next) children.push(next)
      continue
    }
    const next = mapConditions(child, fn)
    if (next.children.length > 0) children.push(next)
  }
  return { type: 'group', combinator: group.combinator, children }
}

export function siblingColumns(group: FilterGroupDraft) {
  return group.children.flatMap((child) => (child.type === 'condition' && child.column ? [child.column] : []))
}

export function newConditionDraft(column: string, op: string): FilterConditionDraft {
  return { type: 'condition', id: nextRuleId('f'), column, op }
}

export function newGroupDraft(combinator: FilterCombinator, child: FilterNodeDraft): FilterGroupDraft {
  return { type: 'group', id: nextRuleId('g'), combinator, children: [child] }
}

export function isEmptyFilterDraft(group: FilterGroupDraft) {
  return group.children.length === 0
}
