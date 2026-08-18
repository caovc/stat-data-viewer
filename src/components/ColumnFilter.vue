<script setup lang="ts">
import { computed, shallowRef, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { Button, Flex, Segmented } from 'antdv-next'
import * as api from '../api'
import type { DistinctValue, FilterSpec } from '../types'
import { columnFilterKind, defaultFilterOp, filterModeOf, type FilterMode } from '../utils/columnFilter'
import FilterConditionForm from './filters/FilterConditionForm.vue'
import FilterValueList, { type FilterValueOption } from './filters/FilterValueList.vue'

const props = defineProps<{
  column: string
  storageType: string
  displayFormat?: string | null
  isDatetime?: boolean
  existing?: FilterSpec | null
  table?: string | null
  pageValues?: DistinctValue[]
  formatValue?: (raw: string) => string
  active?: boolean
}>()

const emit = defineEmits<{
  apply: [spec: FilterSpec]
  clear: []
  cancel: []
}>()

const { t } = useI18n()
const kind = computed(() => columnFilterKind({
  storageType: props.storageType,
  displayFormat: props.displayFormat,
  isDatetime: props.isDatetime,
}))

const mode = shallowRef<FilterMode>(filterModeOf(props.existing))
const op = shallowRef(props.existing && props.existing.op !== 'in' && props.existing.op !== 'not_in'
  ? props.existing.op
  : defaultFilterOp(kind.value))
const value = shallowRef(props.existing?.value ?? '')
const value2 = shallowRef(props.existing?.value2 ?? '')
const includeEmpty = shallowRef(Boolean(props.existing?.includeNull))
const selected = shallowRef<string[]>(props.existing?.values ? [...props.existing.values] : [])
const options = shallowRef<FilterValueOption[]>([])
const emptyCount = shallowRef(0)
const truncated = shallowRef(false)
const loading = shallowRef(false)
const loaded = shallowRef(false)

const modeOptions = computed(() => [
  { label: t('filter.valueList'), value: 'values' },
  { label: t('filter.condition'), value: 'condition' },
])

function displayOf(item: DistinctValue) {
  if (item.value == null) return t('filter.empty')
  const labeled = props.formatValue?.(item.value)
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
  if (loaded.value || loading.value) return
  loading.value = true
  try {
    if (props.table) {
      useDistinct(await api.columnDistinct({ table: props.table, column: props.column }))
      return
    }
    useDistinct({
      values: props.pageValues ?? [],
      emptyCount: (props.pageValues ?? []).find((item) => item.value == null)?.count ?? 0,
      truncated: true,
    })
  } finally {
    loaded.value = true
    loading.value = false
  }
}

watch(
  () => props.active && mode.value === 'values',
  (shouldLoad) => {
    if (shouldLoad) void loadDistinct()
  },
  { immediate: true },
)

function onModeChange(next: string | number) {
  mode.value = next === 'values' ? 'values' : 'condition'
  if (mode.value === 'values') void loadDistinct()
}

function apply() {
  if (mode.value === 'values') {
    emit('apply', {
      column: props.column,
      op: 'in',
      values: selected.value,
      includeNull: includeEmpty.value,
    })
    return
  }
  emit('apply', {
    column: props.column,
    op: op.value,
    value: value.value || undefined,
    value2: value2.value || undefined,
    includeNull: includeEmpty.value || undefined,
  })
}
</script>

<template>
  <Flex vertical gap="small" class="filter-form">
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
      :kind="kind"
      :display-format="displayFormat"
      @apply="apply"
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
    <Flex justify="space-between" gap="small">
      <Button type="link" class="clear-btn" @click="emit('clear')">{{ t('filter.clear') }}</Button>
      <Flex gap="small">
        <Button @click="emit('cancel')">{{ t('filter.cancel') }}</Button>
        <Button type="primary" @click="apply">{{ t('filter.apply') }}</Button>
      </Flex>
    </Flex>
  </Flex>
</template>

<style scoped>
.filter-form {
  width: 280px;
  overflow: visible;
}

.clear-btn {
  padding-inline: 0;
}
</style>
