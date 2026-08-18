<script setup lang="ts">
import { computed, shallowRef } from 'vue'
import { useI18n } from 'vue-i18n'
import { Checkbox, Empty, Flex, Input, Spin, TypographyText } from 'antdv-next'

export interface FilterValueOption {
  value: string
  label: string
  count: number
}

const props = defineProps<{
  options: FilterValueOption[]
  emptyCount: number
  truncated: boolean
  loading: boolean
}>()

const selected = defineModel<string[]>('selected', { required: true })
const includeEmpty = defineModel<boolean>('includeEmpty', { required: true })

const { t } = useI18n()
const search = shallowRef('')

const filtered = computed(() => {
  const q = search.value.trim().toLowerCase()
  if (!q) return props.options
  return props.options.filter((item) => item.label.toLowerCase().includes(q) || item.value.toLowerCase().includes(q))
})

const visibleValues = computed(() => filtered.value.map((item) => item.value))
const allVisibleSelected = computed(() =>
  visibleValues.value.length > 0 && visibleValues.value.every((value) => selected.value.includes(value)),
)
const someVisibleSelected = computed(() =>
  visibleValues.value.some((value) => selected.value.includes(value)) && !allVisibleSelected.value,
)

function toggleAll(checked: boolean) {
  if (checked) {
    selected.value = [...new Set([...selected.value, ...visibleValues.value])]
    return
  }
  const hide = new Set(visibleValues.value)
  selected.value = selected.value.filter((value) => !hide.has(value))
}

function toggle(value: string, checked: boolean) {
  if (checked) {
    if (!selected.value.includes(value)) selected.value = [...selected.value, value]
    return
  }
  selected.value = selected.value.filter((item) => item !== value)
}
</script>

<template>
  <Flex vertical gap="small" class="value-list">
    <Input
      v-model:value="search"
      allow-clear
      :placeholder="t('filter.searchValues')"
    />
    <Spin :spinning="loading">
      <Flex vertical gap="small" class="value-body">
        <Checkbox
          v-if="filtered.length > 0"
          :checked="allVisibleSelected"
          :indeterminate="someVisibleSelected"
          @change="toggleAll($event.target.checked)"
        >
          {{ t('filter.selectAll') }}
        </Checkbox>
        <Checkbox v-model:checked="includeEmpty">
          <Flex align="center" justify="space-between" class="value-row">
            <span>{{ t('filter.empty') }}</span>
            <TypographyText type="secondary" class="value-count">{{ emptyCount }}</TypographyText>
          </Flex>
        </Checkbox>
        <Empty v-if="!loading && filtered.length === 0" :description="t('filter.noValues')" />
        <div v-else class="value-scroll">
          <Checkbox
            v-for="item in filtered"
            :key="item.value"
            :checked="selected.includes(item.value)"
            @change="toggle(item.value, $event.target.checked)"
          >
            <Flex align="center" justify="space-between" class="value-row">
              <TypographyText :ellipsis="{ tooltip: item.label }" class="value-label">{{ item.label }}</TypographyText>
              <TypographyText type="secondary" class="value-count">{{ item.count }}</TypographyText>
            </Flex>
          </Checkbox>
        </div>
        <TypographyText v-if="truncated" type="secondary">{{ t('filter.truncated', { n: options.length }) }}</TypographyText>
      </Flex>
    </Spin>
  </Flex>
</template>

<style scoped>
.value-body {
  min-height: 120px;
}

.value-scroll {
  display: flex;
  max-height: 220px;
  flex-direction: column;
  gap: 6px;
  overflow: auto;
}

.value-row {
  width: 100%;
  min-width: 0;
}

.value-label {
  flex: 1;
  min-width: 0;
}

.value-count {
  flex: 0 0 auto;
  margin-left: 8px;
  font-variant-numeric: tabular-nums;
}

:deep(.ant-checkbox-wrapper) {
  align-items: flex-start;
  width: 100%;
  margin-inline-start: 0;
}
</style>
