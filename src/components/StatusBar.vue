<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { Button, Flex, Pagination, Progress, Segmented, Select, Tag, Tooltip, TypographyText } from 'antdv-next'
import { storeToRefs } from 'pinia'
import { cancelImport } from '../api'
import { useWorkspaceActions } from '../composables/useWorkspaceActions'
import type { ScrollMode } from '../types'
import { PAGE_SIZE_OPTIONS } from '../utils/infiniteScroll'

const { t } = useI18n()
const { store, currentPage, changePage } = useWorkspaceActions()
const { page, pageSize, scrollMode, loadingMore, dataTab, metadata } = storeToRefs(store)

const importPercent = computed(() => Math.round((dataTab.value?.progress ?? 0) * 100))
const pageSizeOptions = PAGE_SIZE_OPTIONS.map(String)
const chunkOptions = PAGE_SIZE_OPTIONS.map((n) => ({ value: n, label: String(n) }))
const modeOptions = computed(() => [
  { value: 'page', label: t('status.pageMode') },
  { value: 'infinite', label: t('status.infiniteMode') },
])

function showTotal(total: number, range: [number, number]) {
  return t('status.range', { start: range[0], end: range[1], total })
}

function onScrollMode(value: string | number) {
  void store.setScrollMode(value as ScrollMode)
}

function onChunk(value: unknown) {
  void store.setPageSize(Number(value))
}
</script>

<template>
  <Flex class="status-bar" align="center" justify="space-between" gap="middle">
    <Flex align="center" gap="small" class="status-left">
      <Tooltip :title="t('status.modeHint')">
        <Segmented
          size="small"
          :value="scrollMode"
          :options="modeOptions"
          :disabled="!page"
          @change="onScrollMode"
        />
      </Tooltip>
      <Pagination
        v-if="scrollMode === 'page'"
        size="small"
        :current="currentPage"
        :page-size="pageSize"
        :total="page?.totalRows ?? 0"
        :disabled="!page"
        :show-size-changer="true"
        :page-size-options="pageSizeOptions"
        :show-total="showTotal"
        @change="changePage"
      />
      <template v-else>
        <TypographyText type="secondary">{{ t('status.chunk') }}</TypographyText>
        <Select
          size="small"
          class="chunk-select"
          :value="pageSize"
          :options="chunkOptions"
          :disabled="!page"
          @change="onChunk"
        />
        <TypographyText v-if="page" type="secondary">
          {{ loadingMore ? t('status.loadingMore') : t('status.loaded', { n: page.rows.length, total: page.totalRows }) }}
        </TypographyText>
      </template>
      <TypographyText v-if="page" type="secondary">
        {{ t('status.columns', { n: page.columns.length }) }}
      </TypographyText>
    </Flex>

    <Flex align="center" gap="small" class="status-right">
      <template v-if="dataTab?.importing">
        <Progress
          type="line"
          size="small"
          :percent="importPercent"
          :style="{ width: '140px', margin: 0 }"
        />
        <Button size="small" danger @click="cancelImport(dataTab.jobId)">{{ t('status.cancel') }}</Button>
      </template>
      <TypographyText v-else-if="dataTab?.error" type="danger">
        {{ dataTab.error }}
      </TypographyText>
      <template v-else-if="dataTab">
        <Tag v-if="metadata?.fileFormat" bordered>{{ metadata.fileFormat }}</Tag>
        <Tag v-if="metadata?.encoding" bordered>{{ metadata.encoding }}</Tag>
        <Tag v-if="metadata && !metadata.importComplete" color="warning">{{ t('status.partial') }}</Tag>
      </template>
    </Flex>
  </Flex>
</template>

<style scoped>
.status-bar {
  width: 100%;
  min-width: 0;
}

.status-left,
.status-right {
  min-width: 0;
}

.status-left :deep(.ant-pagination) {
  flex-wrap: wrap;
}

.chunk-select {
  width: 88px;
}
</style>
