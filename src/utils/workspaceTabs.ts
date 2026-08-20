import type { DataTab, WorkspaceTab } from '../types'

export function fileNameFromPath(path: string, fallback = 'dataset'): string {
  const name = path.split(/[\\/]/).filter(Boolean).pop()
  return name || fallback
}

export function normalizeFilePath(path: string): string {
  return path.replace(/\\/g, '/').replace(/\/+$/, '')
}

export function sameFilePath(a: string, b: string): boolean {
  return normalizeFilePath(a) === normalizeFilePath(b)
}

export function findDataTabByPath(tabs: WorkspaceTab[], path: string): DataTab | undefined {
  return tabs.find((tab): tab is DataTab => tab.kind === 'data' && sameFilePath(tab.path, path))
}

export function releasedDataTableName(
  closed: WorkspaceTab,
  remaining: WorkspaceTab[],
): string | null {
  if (closed.kind !== 'data') return null
  const stillOpen = remaining.some(
    (tab) => tab.kind === 'data' && tab.tableName === closed.tableName,
  )
  return stillOpen ? null : closed.tableName
}

export function omitTableMeta<T>(metadataByTable: Record<string, T>, tableName: string): Record<string, T> {
  if (!(tableName in metadataByTable)) return metadataByTable
  const next = { ...metadataByTable }
  delete next[tableName]
  return next
}
