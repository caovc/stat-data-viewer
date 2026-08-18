<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { UndoOutlined } from '@antdv-next/icons'
import { Button, Drawer, Empty, TypographyParagraph } from 'antdv-next'
import { storeToRefs } from 'pinia'
import ColumnSettingsList from './columns/ColumnSettingsList.vue'
import { useWorkspace } from '../stores/workspace'
import type { ColumnPin, ColumnSetting } from '../types'
import { displayColumnNames, isDefaultColumnLayout, mergeColumnOrder, pinOf } from '../utils/columnLayout'
import { typeFieldsOf } from '../utils/columnType'

const { t } = useI18n()
const store = useWorkspace()
const { metadata, page, hidden, columnOrder, pinnedStart, pinnedEnd, columnWidths, showColumns } = storeToRefs(store)

const sourceColumns = computed(() => {
  if (store.active?.kind === 'sql') return page.value?.columns ?? []
  return metadata.value?.variables ?? page.value?.columns ?? []
})

const sourceNames = computed(() => sourceColumns.value.map((item) => item.name))

const items = computed<ColumnSetting[]>(() => {
  const names = displayColumnNames(
    mergeColumnOrder(columnOrder.value, sourceNames.value),
    pinnedStart.value,
    pinnedEnd.value,
  )
  const byName = new Map(sourceColumns.value.map((item) => [item.name, item]))
  const hiddenSet = new Set(hidden.value)
  return names.map((name) => {
    const col = byName.get(name)
    return {
      name,
      label: col?.label ?? null,
      visible: !hiddenSet.has(name),
      pin: pinOf(name, pinnedStart.value, pinnedEnd.value),
      ...typeFieldsOf(col),
    }
  })
})

const canReset = computed(() => !isDefaultColumnLayout({
  names: sourceNames.value,
  order: columnOrder.value,
  hidden: hidden.value,
  pinnedStart: pinnedStart.value,
  pinnedEnd: pinnedEnd.value,
  widths: columnWidths.value,
}))

async function onToggle(name: string, visible: boolean) {
  const hiddenSet = new Set(hidden.value)
  if (visible) hiddenSet.delete(name)
  else hiddenSet.add(name)
  await store.setHidden([...hiddenSet])
}

function onPin(name: string, pin: ColumnPin) {
  store.pinColumn(name, pin)
}

function onReorder(from: string, to: string) {
  store.reorderColumns(from, to)
}
</script>

<template>
  <Drawer
    v-model:open="showColumns"
    :title="t('columns.title')"
    placement="right"
    :size="420"
  >
    <template #extra>
      <Button :disabled="!canReset" @click="store.resetColumnLayout()">
        <template #icon><UndoOutlined /></template>
        {{ t('columns.reset') }}
      </Button>
    </template>
    <TypographyParagraph type="secondary">
      {{ t('columns.hint') }}
    </TypographyParagraph>
    <Empty v-if="items.length === 0" :description="t('columns.empty')" />
    <ColumnSettingsList
      v-else
      :items="items"
      @toggle="onToggle"
      @pin="onPin"
      @reorder="onReorder"
    />
  </Drawer>
</template>
