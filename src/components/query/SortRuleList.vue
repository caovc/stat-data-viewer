<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { DeleteOutlined, HolderOutlined } from '@antdv-next/icons'
import { Button, Segmented, Select, theme } from 'antdv-next'
import { usePointerReorder } from '../../composables/usePointerReorder'
import { moveById } from '../../utils/columnLayout'
import type { QueryColumn, SortDraft } from '../../utils/queryRules'

const items = defineModel<SortDraft[]>({ required: true })

const props = defineProps<{
  columns: QueryColumn[]
}>()

const { t } = useI18n()
const { token } = theme.useToken()
const { dragId, overId, onHandlePointerDown } = usePointerReorder((from, to) => {
  items.value = moveById(items.value, from, to, (item) => item.id)
})

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
</script>

<template>
  <div class="sort-list">
    <div
      v-for="(item, index) in items"
      :key="item.id"
      class="sort-row"
      :class="{ dragging: dragId === item.id, 'drag-over': overId === item.id && dragId !== item.id }"
      :data-reorder-id="item.id"
    >
      <button
        class="drag-handle"
        type="button"
        :aria-label="t('query.reorder')"
        @pointerdown="onHandlePointerDown(item.id, $event)"
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
  </div>
</template>

<style scoped>
.sort-list {
  display: flex;
  flex-direction: column;
  gap: 8px;
}

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
  touch-action: none;
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
