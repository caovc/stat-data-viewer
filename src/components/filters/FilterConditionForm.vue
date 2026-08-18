<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { Checkbox, DatePicker, DateRangePicker, Flex, Input, InputNumber, Select, TimePicker } from 'antdv-next'
import { dateTimePickerKind } from '../../utils/columnType'
import {
  filterNeedsRange,
  filterNeedsValue,
  filterOpLabelKey,
  filterOpsFor,
  type ColumnTypeKind,
} from '../../utils/columnFilter'

const props = defineProps<{
  kind: ColumnTypeKind
  displayFormat?: string | null
  popup?: 'form' | 'body'
}>()

const emit = defineEmits<{
  apply: []
}>()

const op = defineModel<string>('op', { required: true })
const value = defineModel<string>('value', { required: true })
const value2 = defineModel<string>('value2', { required: true })
const includeEmpty = defineModel<boolean>('includeEmpty', { required: true })

const { t } = useI18n()
const pickerKind = computed(() => dateTimePickerKind(props.displayFormat))
const needsValue = computed(() => filterNeedsValue(op.value))
const needsRange = computed(() => filterNeedsRange(op.value))
const showIncludeEmpty = computed(() => needsValue.value)

const opOptions = computed(() =>
  filterOpsFor(props.kind).map((item) => ({
    label: t(filterOpLabelKey(props.kind, item)),
    value: item,
  })),
)

const dateValue = computed<string | null>({
  get() {
    return value.value || null
  },
  set(next) {
    value.value = next ?? ''
  },
})

const dateValue2 = computed<string | null>({
  get() {
    return value2.value || null
  },
  set(next) {
    value2.value = next ?? ''
  },
})

const rangeValue = computed<[string, string] | null>({
  get() {
    if (!value.value && !value2.value) return null
    return [value.value, value2.value]
  },
  set(next) {
    value.value = next?.[0] ?? ''
    value2.value = next?.[1] ?? ''
  },
})

const numberValue = computed<number | null>({
  get() {
    if (value.value.trim() === '') return null
    const parsed = Number(value.value)
    return Number.isFinite(parsed) ? parsed : null
  },
  set(next) {
    value.value = next == null ? '' : String(next)
  },
})

const numberValue2 = computed<number | null>({
  get() {
    if (value2.value.trim() === '') return null
    const parsed = Number(value2.value)
    return Number.isFinite(parsed) ? parsed : null
  },
  set(next) {
    value2.value = next == null ? '' : String(next)
  },
})

function popupContainer(node: HTMLElement) {
  if (props.popup === 'body') return document.body
  return (node.closest('.filter-form') as HTMLElement | null) ?? document.body
}
</script>

<template>
  <Flex vertical gap="small">
    <Select v-model:value="op" :options="opOptions" :get-popup-container="popupContainer" />
    <template v-if="needsValue">
      <template v-if="kind === 'string'">
        <Input
          v-if="!needsRange"
          v-model:value="value"
          :placeholder="t('filter.value')"
          @press-enter="emit('apply')"
        />
      </template>
      <template v-else-if="kind === 'datetime'">
        <DateRangePicker
          v-if="needsRange && pickerKind === 'date'"
          v-model:value="rangeValue"
          value-format="YYYY-MM-DD"
          :placeholder="[t('filter.valueFrom'), t('filter.valueTo')]"
          :get-popup-container="popupContainer"
        />
        <DateRangePicker
          v-else-if="needsRange && pickerKind === 'datetime'"
          v-model:value="rangeValue"
          show-time
          value-format="YYYY-MM-DD HH:mm:ss"
          :placeholder="[t('filter.valueFrom'), t('filter.valueTo')]"
          :get-popup-container="popupContainer"
        />
        <Flex v-else-if="needsRange" gap="small">
          <TimePicker
            v-model:value="dateValue"
            value-format="HH:mm:ss"
            :placeholder="t('filter.valueFrom')"
            :get-popup-container="popupContainer"
          />
          <TimePicker
            v-model:value="dateValue2"
            value-format="HH:mm:ss"
            :placeholder="t('filter.valueTo')"
            :get-popup-container="popupContainer"
          />
        </Flex>
        <DatePicker
          v-else-if="pickerKind === 'date'"
          v-model:value="dateValue"
          value-format="YYYY-MM-DD"
          :placeholder="t('filter.value')"
          :get-popup-container="popupContainer"
        />
        <DatePicker
          v-else-if="pickerKind === 'datetime'"
          v-model:value="dateValue"
          show-time
          value-format="YYYY-MM-DD HH:mm:ss"
          :placeholder="t('filter.value')"
          :get-popup-container="popupContainer"
        />
        <TimePicker
          v-else
          v-model:value="dateValue"
          value-format="HH:mm:ss"
          :placeholder="t('filter.value')"
          :get-popup-container="popupContainer"
        />
      </template>
      <template v-else>
        <Flex v-if="needsRange" gap="small">
          <InputNumber
            v-model:value="numberValue"
            class="filter-number"
            :precision="kind === 'integer' ? 0 : undefined"
            :placeholder="t('filter.valueFrom')"
            @press-enter="emit('apply')"
          />
          <InputNumber
            v-model:value="numberValue2"
            class="filter-number"
            :precision="kind === 'integer' ? 0 : undefined"
            :placeholder="t('filter.valueTo')"
            @press-enter="emit('apply')"
          />
        </Flex>
        <InputNumber
          v-else
          v-model:value="numberValue"
          class="filter-number"
          :precision="kind === 'integer' ? 0 : undefined"
          :placeholder="t('filter.value')"
          @press-enter="emit('apply')"
        />
      </template>
    </template>
    <Checkbox v-if="showIncludeEmpty" v-model:checked="includeEmpty">
      {{ t('filter.includeEmpty') }}
    </Checkbox>
  </Flex>
</template>

<style scoped>
.filter-number {
  width: 100%;
}

:deep(.ant-picker),
:deep(.ant-input-number) {
  width: 100%;
}
</style>
