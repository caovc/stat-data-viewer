<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  ExportOutlined,
  FilterOutlined,
  ReloadOutlined,
  TableOutlined,
} from '@antdv-next/icons'
import { Badge, Button, Flex, Tooltip, theme } from 'antdv-next'
import { storeToRefs } from 'pinia'
import { useWorkspace } from '../stores/workspace'
import { filterCount } from '../utils/queryRules'

const { t } = useI18n()
const { token } = theme.useToken()
const store = useWorkspace()
const { dataTab, page, showQuery, sorts, filters } = storeToRefs(store)

const queryCount = computed(() => sorts.value.length + filterCount(filters.value))
</script>

<template>
  <Flex
    class="view-toolbar"
    align="center"
    gap="small"
    :style="{ borderBottom: `1px solid ${token.colorBorderSecondary}` }"
  >
    <Tooltip :title="t('header.reimport')">
      <Button size="small" :disabled="!dataTab" @click="store.showReimport = true">
        <template #icon><ReloadOutlined /></template>
        {{ t('header.reimport') }}
      </Button>
    </Tooltip>
    <Button size="small" :disabled="!page" @click="store.showExport = true">
      <template #icon><ExportOutlined /></template>
      {{ t('header.export') }}
    </Button>
    <Button size="small" :disabled="!page" @click="store.showColumns = !store.showColumns">
      <template #icon><TableOutlined /></template>
      {{ t('header.columns') }}
    </Button>
    <Tooltip :title="t('header.queryHint')">
      <Badge :count="queryCount" :offset="[-2, 2]">
        <Button
          size="small"
          :disabled="!dataTab"
          :type="showQuery ? 'primary' : 'default'"
          @click="store.showQuery = !store.showQuery"
        >
          <template #icon><FilterOutlined /></template>
          {{ t('header.query') }}
        </Button>
      </Badge>
    </Tooltip>
  </Flex>
</template>

<style scoped>
.view-toolbar {
  flex: none;
  min-width: 0;
  padding: 6px 12px;
}
</style>
