<script setup lang="ts">
import { computed, shallowRef, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { DeleteOutlined } from '@antdv-next/icons'
import { Button, Flex, Segmented, Select, theme } from 'antdv-next'
import * as api from '../../api'
import type { DistinctValue, FilterSpec } from '../../types'
import {
  columnFilterKind,
  defaultFilterOp,
  filterModeOf,
  type FilterMode,
} from '../../utils/columnFilter'
import type { QueryColumn } from '../../utils/queryRules'
import FilterConditionForm from '../filters/FilterConditionForm.vue'
import FilterValueList, { type FilterValueOption } from '../filters/FilterValueList.vue'

const spec = defineModel<FilterSpec>({ required: true })

const props = defineProps<{
  columns: QueryColumn[]
  table?: string | null
  formatValue?: (column: string, raw: string) => string
}>()

const emit = defineEmits<{
  remove: []
}>()

const { t } = useI18n()
const { token } = theme.useToken()

const column = computed(() => props.columns.find((item) => item.name === spec.value.column) ?? null)
const kind = computed(() => columnFilterKind({
  storageType: column.value?.storageType ?? 'string',
  displayFormat: column.value?.displayFormat,
  isDatetime: column.value?.isDatetime,
}))

const mode = shallowRef<FilterMode>(filterModeOf(spec.value))
const op = shallowRef(spec.value.op === 'in' || spec.value.op === 'not_in'
  ? defaultFilterOp(kind.value)
  : spec.value.op)
const value = shallowRef(spec.value.value ?? '')
const value2 = shallowRef(spec.value.value2 ?? '')
const includeEmpty = shallowRef(Boolean(spec.value.includeNull))
const selected = shallowRef<string[]>(spec.value.values ? [...spec.value.values] : [])
const options = shallowRef<FilterValueOption[]>([])
const emptyCount = shallowRef(0)
const truncated = shallowRef(false)
const loading = shallowRef(false)
const loaded = shallowRef(false)

const modeOptions = computed(() => [
  { label: t('filter.valueList'), value: 'values' },
  { label: t('filter.condition'), value: 'condition' },
])

const columnOptions = computed(() =>
  props.columns.map((item) => ({
    value: item.name,
    label: item.label ? `${item.name} · ${item.label}` : item.name,
  })),
)

function displayOf(item: DistinctValue) {
  if (item.value == null) return t('filter.empty')
  const labeled = props.formatValue?.(spec.value.column, item.value)
  if (labeled && labeled !== item.value) return labeled
  return item.label || item.value
}

function useDistinct(result: { values: DistinctValue[]; emptyCount: number; truncated: boolean }) {
  emptyCount.value = result.emptyCount
  truncated.value = result.truncated
  options.value = result.values
    .filter((item): item is DistinctValue & { value: string } => item.value != null)
    .map((item) => ({
      value: item.value,
      label: displayOf(item),
      count: item.count,
    }))
}

async function loadDistinct() {
  if (loaded.value || loading.value || !spec.value.column) return
  loading.value = true
  try {
    if (props.table) {
      useDistinct(await api.columnDistinct({ table: props.table, column: spec.value.column }))
    }
  } finally {
    loaded.value = true
    loading.value = false
  }
}

function resetValues() {
  loaded.value = false
  options.value = []
  emptyCount.value = 0
  truncated.value = false
  selected.value = []
  includeEmpty.value = false
}

function onColumnChange(name: string) {
  spec.value = { ...spec.value, column: name }
  op.value = defaultFilterOp(kind.value)
  value.value = ''
  value2.value = ''
  resetValues()
  if (mode.value === 'values') void loadDistinct()
}

function onModeChange(next: string | number) {
  mode.value = next === 'values' ? 'values' : 'condition'
  if (mode.value === 'values') void loadDistinct()
}

function writeSpec() {
  if (mode.value === 'values') {
    spec.value = {
      column: spec.value.column,
      op: 'in',
      values: [...selected.value],
      includeNull: includeEmpty.value,
    }
    return
  }
  spec.value = {
    column: spec.value.column,
    op: op.value,
    value: value.value || undefined,
    value2: value2.value || undefined,
    includeNull: includeEmpty.value || undefined,
  }
}

watch([mode, op, value, value2, includeEmpty, selected], writeSpec)
watch(
  () => mode.value === 'values',
  (shouldLoad) => {
    if (shouldLoad) void loadDistinct()
  },
  { immediate: true },
)
</script>

<template>
  <Flex vertical gap="small" class="filter-form rule">
    <Flex align="center" gap="small">
      <Select
        :value="spec.column"
        show-search
        class="column-select"
        :options="columnOptions"
        option-filter-prop="label"
        :placeholder="t('query.column')"
        @change="onColumnChange"
      />
      <Button type="text" :aria-label="t('query.remove')" @click="emit('remove')">
        <template #icon><DeleteOutlined /></template>
      </Button>
    </Flex>
    <Segmented
      :value="mode"
      block
      size="small"
      :options="modeOptions"
      @change="onModeChange"
    />
    <FilterConditionForm
      v-if="mode === 'condition'"
      v-model:op="op"
      v-model:value="value"
      v-model:value2="value2"
      v-model:include-empty="includeEmpty"
      popup="body"
      :kind="kind"
      :display-format="column?.displayFormat"
    />
    <FilterValueList
      v-else
      v-model:selected="selected"
      v-model:include-empty="includeEmpty"
      :options="options"
      :empty-count="emptyCount"
      :truncated="truncated"
      :loading="loading"
    />
  </Flex>
</template>

<style scoped>
.rule {
  padding: 10px;
  border-radius: 10px;
  background: v-bind('token.colorFillQuaternary');
}

.column-select {
  flex: 1;
  min-width: 0;
}
</style>
