import type { HeaderMode } from '../types'

export type ColumnTypeKind = 'string' | 'integer' | 'number' | 'datetime'

export interface ColumnTypeInput {
  storageType?: string | null
  displayFormat?: string | null
  isDatetime?: boolean | null
}

const DATE_HINTS = [
  'DATE',
  'DATETIME',
  'TIME',
  'TOD',
  'YYMM',
  'MMYY',
  'MONYY',
  'DDMM',
  'MMDD',
  'ADATE',
  'EDATE',
  'SDATE',
  'JDATE',
  'YMDHMS',
  'DTIME',
  'HHMM',
  'HOUR',
  'WEEKDATE',
  'WORDDATE',
]

export function looksLikeDatetime(format?: string | null) {
  if (!format) return false
  const trimmed = format.trim()
  if (!trimmed) return false
  if (/^%t[cCdDwWmMqQhHyY]/i.test(trimmed)) return true
  const key = trimmed.replace(/[^A-Za-z%]/g, '').toUpperCase()
  return DATE_HINTS.some((hint) => key.startsWith(hint))
}

export type DateTimePickerKind = 'date' | 'datetime' | 'time'

export function dateTimePickerKind(format?: string | null): DateTimePickerKind {
  if (!format) return 'date'
  const trimmed = format.trim()
  if (!trimmed) return 'date'
  if (/^%tC/i.test(trimmed) || /^%tc/i.test(trimmed)) return 'datetime'
  const key = trimmed.replace(/[^A-Za-z%]/g, '').toUpperCase()
  if (
    key.startsWith('DATETIME')
    || key.startsWith('YMDHMS')
    || key.startsWith('B8601DT')
    || key.startsWith('E8601DT')
    || key.startsWith('DTDATE')
  ) {
    return 'datetime'
  }
  if (['TIME', 'TOD', 'HHMM', 'HOUR', 'MMSS', 'TIMEAMPM', 'DTIME'].some((hint) => key.startsWith(hint))) {
    return 'time'
  }
  return 'date'
}

export function columnTypeKind(input: ColumnTypeInput): ColumnTypeKind {
  if (input.isDatetime || looksLikeDatetime(input.displayFormat)) return 'datetime'
  const storage = (input.storageType ?? '').toLowerCase()
  if (storage === 'string' || storage === 'varchar' || storage === 'utf8' || storage.includes('str')) {
    return 'string'
  }
  if (storage === 'int32' || storage === 'int64' || storage === 'int' || storage === 'integer') {
    return 'integer'
  }
  return 'number'
}

export function typeFieldsOf(col?: ColumnTypeInput | null) {
  return {
    storageType: col?.storageType ?? 'string',
    displayFormat: col?.displayFormat ?? null,
    isDatetime: Boolean(col?.isDatetime),
  }
}

export function headerDisplay(mode: HeaderMode, name: string, label: string | null) {
  const resolved = label?.trim() || null
  if (mode === 'name') return { primary: name, secondary: null as string | null }
  if (mode === 'label') return { primary: resolved ?? name, secondary: null }
  return { primary: name, secondary: resolved }
}
