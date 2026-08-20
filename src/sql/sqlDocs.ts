import type { SqlColumn, SqlTable } from './catalog.ts'

export function tableSourcePath(table: SqlTable): string | null {
  return table.path || table.title || null
}

export function tableHoverMarkdown(table: SqlTable): string {
  const lines = [`**${table.name}**`]
  if (table.fileLabel) lines.push(table.fileLabel)
  const bits = [
    table.rowCount != null ? `${table.rowCount.toLocaleString()} rows` : null,
    table.varCount != null ? `${table.varCount} variables` : null,
  ].filter(Boolean)
  if (bits.length) lines.push(bits.join(' · '))
  const source = tableSourcePath(table)
  if (source) lines.push(`Source\n\n\`${source}\``)
  return lines.join('\n\n')
}

export function columnHoverMarkdown(column: SqlColumn, table?: SqlTable): string {
  const lines = [`**${column.name}**`]
  if (column.label) lines.push(column.label)
  const bits = [`\`${column.storageType}\``]
  if (column.displayFormat) bits.push(`format ${column.displayFormat}`)
  if (column.measure) bits.push(column.measure)
  lines.push(bits.join(' · '))
  lines.push(`Table: \`${column.table}\``)
  const source = table ? tableSourcePath(table) : null
  if (source) lines.push(`Source\n\n\`${source}\``)
  if (column.missingRules) lines.push(`Missing: ${column.missingRules}`)
  return lines.join('\n\n')
}

export function shouldHoverTable(textAfterToken: string): boolean {
  return !/^\s*\./.test(textAfterToken)
}
