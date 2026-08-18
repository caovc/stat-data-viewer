<script setup lang="ts">
import { computed, shallowRef } from 'vue'
import { useI18n } from 'vue-i18n'
import { SearchOutlined } from '@antdv-next/icons'
import { Empty, Flex, Input, Table, Tag, TypographyText, theme } from 'antdv-next'
import { storeToRefs } from 'pinia'
import ColumnTypeIcon from './columns/ColumnTypeIcon.vue'
import { useWorkspace } from '../stores/workspace'

const { t } = useI18n()
const { token } = theme.useToken()
const store = useWorkspace()
const { metadata, dataTab } = storeToRefs(store)
const query = shallowRef('')

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
  <aside class="meta-panel">
    <Flex class="meta-head" align="center" justify="space-between">
      <TypographyText strong>{{ t('meta.variables') }}</TypographyText>
      <Tag v-if="metadata" bordered>
        {{ metadata.variables.length }}
      </Tag>
    </Flex>
    <div v-if="!dataTab" class="meta-empty">
      <Empty
        :description="t('meta.empty')"
      />
    </div>
    <Flex v-else vertical class="meta-body" :style="{ gap: `${token.paddingSM}px` }">
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
        :scroll="{ y: metadata?.valueLabels.length ? 260 : 'calc(100vh - 260px)' }"
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
          :scroll="{ y: 220 }"
        />
      </template>
    </Flex>
  </aside>
</template>

<style scoped>
.meta-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  min-width: 0;
}

.meta-head {
  padding: 10px 12px;
  border-bottom: 1px solid v-bind('token.colorBorderSecondary');
}

.meta-body,
.meta-empty {
  flex: 1;
  min-height: 0;
  padding: 12px;
  overflow: auto;
}

.name-cell,
.cell-ellipsis {
  min-width: 0;
}

.name-cell .cell-ellipsis {
  flex: 1;
}
</style>
