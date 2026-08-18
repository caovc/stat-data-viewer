<script setup lang="ts">
import { computed, shallowRef } from 'vue'
import { useI18n } from 'vue-i18n'
import { DeleteOutlined, HolderOutlined } from '@antdv-next/icons'
import { Button, Segmented, Select, theme } from 'antdv-next'
import type { QueryColumn, SortDraft } from '../../utils/queryRules'

const items = defineModel<SortDraft[]>({ required: true })

const props = defineProps<{
  columns: QueryColumn[]
}>()

const { t } = useI18n()
const { token } = theme.useToken()
const dragId = shallowRef<string | null>(null)
const overId = shallowRef<string | null>(null)

const dirOptions = computed(() => [
  { label: t('query.asc'), value: 'asc' },
  { label: t('query.desc'), value: 'desc' },
])

const columnOptions = computed(() =>
  props.columns.map((item) => ({
    value: item.name,
    label: item.label ? `${item.name} · ${item.label}` : item.name,
  })),
)

function optionsFor(id: string) {
  const used = new Set(items.value.filter((item) => item.id !== id).map((item) => item.column))
  return columnOptions.value.filter((item) => !used.has(item.value) || items.value.find((row) => row.id === id)?.column === item.value)
}

function setColumn(id: string, column: string) {
  items.value = items.value.map((item) => (item.id === id ? { ...item, column } : item))
}

function setDir(id: string, desc: boolean) {
  items.value = items.value.map((item) => (item.id === id ? { ...item, desc } : item))
}

function remove(id: string) {
  items.value = items.value.filter((item) => item.id !== id)
}

function onDragStart(event: DragEvent, id: string) {
  event.dataTransfer?.setData('text/plain', id)
  dragId.value = id
}

function onDrop(id: string) {
  const from = items.value.findIndex((item) => item.id === dragId.value)
  const to = items.value.findIndex((item) => item.id === id)
  if (from >= 0 && to >= 0 && from !== to) {
    const next = [...items.value]
    const [moved] = next.splice(from, 1)
    next.splice(to, 0, moved)
    items.value = next
  }
  dragId.value = null
  overId.value = null
}
</script>

<template>
  <Flex vertical gap="small">
    <div
      v-for="(item, index) in items"
      :key="item.id"
      class="sort-row"
      :class="{ dragging: dragId === item.id, 'drag-over': overId === item.id && dragId !== item.id }"
      @dragover.prevent="overId = item.id"
      @drop.prevent="onDrop(item.id)"
    >
      <button
        class="drag-handle"
        type="button"
        draggable="true"
        :aria-label="t('query.reorder')"
        @dragstart="onDragStart($event, item.id)"
        @dragend="dragId = null; overId = null"
      >
        <HolderOutlined />
      </button>
      <span class="sort-ord">{{ index + 1 }}</span>
      <Select
        :value="item.column"
        show-search
        class="column-select"
        :options="optionsFor(item.id)"
        option-filter-prop="label"
        :placeholder="t('query.column')"
        @change="setColumn(item.id, String($event))"
      />
      <Segmented
        size="small"
        :value="item.desc ? 'desc' : 'asc'"
        :options="dirOptions"
        @change="setDir(item.id, $event === 'desc')"
      />
      <Button type="text" :aria-label="t('query.remove')" @click="remove(item.id)">
        <template #icon><DeleteOutlined /></template>
      </Button>
    </div>
  </Flex>
</template>

<style scoped>
.sort-row {
  display: grid;
  grid-template-columns: 28px 18px minmax(0, 1fr) auto auto;
  gap: 8px;
  align-items: center;
  padding: 8px;
  border: 1px solid transparent;
  border-radius: 10px;
  background: v-bind('token.colorFillQuaternary');
}

.sort-row.drag-over {
  border-color: v-bind('token.colorPrimary');
  background: v-bind('token.colorPrimaryBg');
}

.sort-row.dragging {
  opacity: 0.45;
}

.drag-handle {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  padding: 0;
  border: 0;
  color: v-bind('token.colorTextTertiary');
  background: transparent;
  cursor: grab;
}

.drag-handle:active {
  cursor: grabbing;
}

.sort-ord {
  color: v-bind('token.colorTextTertiary');
  font-variant-numeric: tabular-nums;
  text-align: center;
}

.column-select {
  min-width: 0;
}
</style>
