<script setup lang="ts">
import { computed, shallowRef } from 'vue'
import { useI18n } from 'vue-i18n'
import { SearchOutlined } from '@antdv-next/icons'
import { Drawer, Empty, Flex, Input, Table, Tag, TypographyText } from 'antdv-next'
import { storeToRefs } from 'pinia'
import ColumnTypeIcon from './columns/ColumnTypeIcon.vue'
import { useWorkspace } from '../stores/workspace'

const { t } = useI18n()
const store = useWorkspace()
const { metadata, dataTab, showVariables } = storeToRefs(store)
const query = shallowRef('')
const panelSize = shallowRef(440)

function onResize(next: number) {
  panelSize.value = Math.min(800, Math.max(280, next))
}

const filteredVariables = computed(() => {
  const items = metadata.value?.variables ?? []
  const q = query.value.trim().toLowerCase()
  if (!q) return items
  return items.filter((item) =>
    [item.name, item.label, item.storageType, item.displayFormat]
      .some((value) => value?.toLowerCase().includes(q)),
  )
})

const labelRows = computed(() =>
  (metadata.value?.valueLabels ?? []).map((item, index) => ({
    key: `${item.labelSet}-${item.numValue ?? item.strValue ?? item.tag}-${index}`,
    labelSet: item.labelSet,
    value: item.numValue ?? item.strValue ?? item.tag ?? '',
    label: item.label,
  })),
)

const variableColumns = computed(() => [
  { title: t('meta.name'), dataIndex: 'name', key: 'name', width: 132, ellipsis: true },
  { title: t('meta.label'), dataIndex: 'label', key: 'label', ellipsis: true },
  { title: t('meta.type'), dataIndex: 'storageType', key: 'storageType', width: 72 },
  { title: t('meta.format'), dataIndex: 'displayFormat', key: 'displayFormat', width: 80, ellipsis: true },
])

const labelColumns = computed(() => [
  { title: t('meta.set'), dataIndex: 'labelSet', key: 'labelSet', ellipsis: true },
  { title: t('meta.value'), dataIndex: 'value', key: 'value', width: 72 },
  { title: t('meta.label'), dataIndex: 'label', key: 'label', ellipsis: true },
])
</script>

<template>
  <Drawer
    v-model:open="showVariables"
    :title="t('meta.variables')"
    placement="left"
    :size="panelSize"
    :max-size="800"
    destroy-on-hidden
    :resizable="{ onResize }"
  >
    <template #extra>
      <Tag v-if="metadata" bordered>
        {{ metadata.variables.length }}
      </Tag>
    </template>
    <div v-if="!dataTab" class="meta-empty">
      <Empty :description="t('meta.empty')" />
    </div>
    <Flex v-else vertical gap="small" class="meta-body">
      <Input
        v-model:value="query"
        allow-clear
        :placeholder="t('meta.search')"
      >
        <template #prefix>
          <SearchOutlined />
        </template>
      </Input>
      <Table
        size="small"
        row-key="name"
        :columns="variableColumns"
        :data-source="filteredVariables"
        :pagination="false"
      >
        <template #bodyCell="{ column, record, text }">
          <Flex v-if="column.key === 'name'" align="center" :gap="6" class="name-cell">
            <ColumnTypeIcon
              :storage-type="record.storageType"
              :display-format="record.displayFormat"
            />
            <TypographyText :ellipsis="{ tooltip: record.name }" class="cell-ellipsis">
              {{ record.name }}
            </TypographyText>
          </Flex>
          <TypographyText
            v-else-if="column.key === 'label'"
            :ellipsis="{ tooltip: record.label || undefined }"
            class="cell-ellipsis"
          >
            {{ record.label }}
          </TypographyText>
          <TypographyText
            v-else-if="column.key === 'storageType'"
            :ellipsis="{ tooltip: text }"
            class="cell-ellipsis"
          >
            {{ text }}
          </TypographyText>
          <TypographyText
            v-else-if="column.key === 'displayFormat'"
            :ellipsis="{ tooltip: text || undefined }"
            class="cell-ellipsis"
          >
            {{ text }}
          </TypographyText>
        </template>
      </Table>
      <template v-if="metadata?.valueLabels.length">
        <TypographyText strong>{{ t('meta.valueLabels') }}</TypographyText>
        <Table
          size="small"
          row-key="key"
          :columns="labelColumns"
          :data-source="labelRows"
          :pagination="false"
        />
      </template>
    </Flex>
  </Drawer>
</template>

<style scoped>
.meta-body,
.meta-empty {
  min-width: 0;
}

.name-cell,
.cell-ellipsis {
  min-width: 0;
}

.name-cell .cell-ellipsis {
  flex: 1;
}
</style>
