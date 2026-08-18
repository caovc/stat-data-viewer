<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { DeleteOutlined, PlusOutlined } from '@antdv-next/icons'
import { Button, Flex, Segmented, theme } from 'antdv-next'
import FilterRuleItem from './FilterRuleItem.vue'
import type { FilterSpec } from '../../types'
import { columnFilterKind, defaultFilterOp } from '../../utils/columnFilter'
import {
  firstUnusedColumn,
  newConditionDraft,
  newGroupDraft,
  siblingColumns,
  type FilterGroupDraft,
  type FilterNodeDraft,
  type QueryColumn,
} from '../../utils/queryRules'

defineOptions({ name: 'FilterGroupEditor' })

const group = defineModel<FilterGroupDraft>({ required: true })

const props = defineProps<{
  columns: QueryColumn[]
  table?: string | null
  formatValue?: (column: string, raw: string) => string
  root?: boolean
}>()

const emit = defineEmits<{
  remove: []
}>()

const { t } = useI18n()
const { token } = theme.useToken()

const combinatorOptions = computed(() => [
  { label: t('query.and'), value: 'and' },
  { label: t('query.or'), value: 'or' },
])

const combinator = computed({
  get: () => group.value.combinator,
  set: (next: string | number) => {
    group.value = { ...group.value, combinator: next === 'or' ? 'or' : 'and' }
  },
})

function nextCondition() {
  const col = firstUnusedColumn(props.columns, siblingColumns(group.value)) ?? props.columns[0]
  if (!col) return null
  return newConditionDraft(col.name, defaultFilterOp(columnFilterKind(col)))
}

function replaceChild(id: string, next: FilterNodeDraft) {
  group.value = {
    ...group.value,
    children: group.value.children.map((child) => (child.id === id ? next : child)),
  }
}

function removeChild(id: string) {
  group.value = {
    ...group.value,
    children: group.value.children.filter((child) => child.id !== id),
  }
}

function addCondition() {
  const child = nextCondition()
  if (!child) return
  group.value = { ...group.value, children: [...group.value.children, child] }
}

function addGroup() {
  const child = nextCondition()
  if (!child) return
  const nested = newGroupDraft(group.value.combinator === 'and' ? 'or' : 'and', child)
  group.value = { ...group.value, children: [...group.value.children, nested] }
}

function updateCondition(id: string, spec: FilterSpec) {
  replaceChild(id, { type: 'condition', id, ...spec })
}

function updateGroup(id: string, next: FilterGroupDraft) {
  replaceChild(id, next)
}
</script>

<template>
  <div class="group" :class="`is-${group.combinator}`">
    <Flex class="group-bar" align="center" justify="space-between" gap="small" wrap="wrap">
      <Segmented
        v-model:value="combinator"
        size="small"
        :options="combinatorOptions"
      />
      <Flex align="center" gap="small">
        <Button size="small" :disabled="columns.length === 0" @click="addCondition">
          <template #icon><PlusOutlined /></template>
          {{ t('query.addCondition') }}
        </Button>
        <Button size="small" :disabled="columns.length === 0" @click="addGroup">
          <template #icon><PlusOutlined /></template>
          {{ t('query.addGroup') }}
        </Button>
        <Button
          v-if="!root"
          type="text"
          size="small"
          :aria-label="t('query.remove')"
          @click="emit('remove')"
        >
          <template #icon><DeleteOutlined /></template>
        </Button>
      </Flex>
    </Flex>
    <div v-if="group.children.length === 0" class="group-empty">
      {{ t('query.filterEmpty') }}
    </div>
    <div v-else class="group-body">
      <template v-for="(child, index) in group.children" :key="child.id">
        <div v-if="index > 0" class="join">{{ group.combinator === 'or' ? t('query.or') : t('query.and') }}</div>
        <FilterRuleItem
          v-if="child.type === 'condition'"
          :model-value="child"
          :columns="columns"
          :table="table"
          :format-value="formatValue"
          @update:model-value="updateCondition(child.id, $event)"
          @remove="removeChild(child.id)"
        />
        <FilterGroupEditor
          v-else-if="child.type === 'group'"
          :model-value="child"
          :columns="columns"
          :table="table"
          :format-value="formatValue"
          @update:model-value="updateGroup(child.id, $event)"
          @remove="removeChild(child.id)"
        />
      </template>
    </div>
  </div>
</template>

<style scoped>
.group {
  min-width: 0;
  padding: 8px;
  border: 1px solid v-bind('token.colorBorderSecondary');
  border-radius: 10px;
  background: v-bind('token.colorBgContainer');
}

.group.is-and {
  border-left: 3px solid v-bind('token.colorPrimary');
}

.group.is-or {
  border-left: 3px solid v-bind('token.colorWarning');
}

.group-bar {
  min-width: 0;
}

.group-empty {
  margin-top: 8px;
  color: v-bind('token.colorTextTertiary');
  font-size: 12px;
}

.group-body {
  display: flex;
  flex-direction: column;
  gap: 8px;
  min-width: 0;
  margin-top: 8px;
  padding-left: 10px;
  border-left: 1px dashed v-bind('token.colorBorder');
}

.join {
  align-self: flex-start;
  padding: 0 6px;
  color: v-bind('token.colorTextSecondary');
  font-size: 11px;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
  letter-spacing: 0.04em;
}

.group-bar :deep(.ant-btn) {
  padding-inline: 8px;
}
</style>
