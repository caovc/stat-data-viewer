<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import {
  CodeOutlined,
  DatabaseOutlined,
  FolderOpenOutlined,
} from '@antdv-next/icons'
import { Button, Divider, Flex, Segmented, Space, Tooltip, TypographyText, theme } from 'antdv-next'
import { storeToRefs } from 'pinia'
import LocaleSwitch from './header/LocaleSwitch.vue'
import ThemeSwitch from './header/ThemeSwitch.vue'
import { useWorkspaceActions } from '../composables/useWorkspaceActions'

const { t } = useI18n()
const { token } = theme.useToken()
const { store, openFiles } = useWorkspaceActions()
const { page, metadata, showSql, headerMode, labelMode } = storeToRefs(store)

const headerOptions = computed(() => [
  { label: t('header.colName'), value: 'name' },
  { label: t('header.colLabel'), value: 'label' },
  { label: t('header.colBoth'), value: 'both' },
])

const labelOptions = computed(() => [
  { label: t('header.labelValue'), value: 'value' },
  { label: t('header.labelLabel'), value: 'label' },
  { label: t('header.labelBoth'), value: 'both' },
])

const hasValueLabels = computed(() => (metadata.value?.valueLabels.length ?? 0) > 0)
</script>

<template>
  <Flex class="header-bar" align="center" justify="space-between" gap="middle">
    <Flex align="center" gap="small" class="brand">
      <span
        class="brand-mark"
        :style="{
          background: token.colorPrimary,
          color: token.colorWhite,
        }"
      >
        <DatabaseOutlined />
      </span>
      <div class="brand-copy">
        <TypographyText strong>{{ t('app.name') }}</TypographyText>
        <TypographyText type="secondary" class="brand-sub">
          {{ t('app.subtitle') }}
        </TypographyText>
      </div>
    </Flex>

    <Space :size="8" wrap>
      <Button type="primary" @click="openFiles">
        <template #icon><FolderOpenOutlined /></template>
        {{ t('header.open') }}
      </Button>
      <Button :type="showSql ? 'primary' : 'default'" @click="store.showSql = !store.showSql">
        <template #icon><CodeOutlined /></template>
        {{ t('header.sql') }}
      </Button>
      <Divider type="vertical" />
      <Tooltip :title="t('header.colModeHint')">
        <span>
          <Segmented
            v-model:value="headerMode"
            size="small"
            :options="headerOptions"
            :disabled="!page"
          />
        </span>
      </Tooltip>
      <Tooltip :title="t('header.labelModeHint')">
        <span>
          <Segmented
            v-model:value="labelMode"
            size="small"
            :options="labelOptions"
            :disabled="!page || !hasValueLabels"
          />
        </span>
      </Tooltip>
      <Divider type="vertical" />
      <ThemeSwitch />
      <LocaleSwitch />
    </Space>
  </Flex>
</template>

<style scoped>
.header-bar {
  width: 100%;
  min-width: 0;
}

.brand-mark {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 32px;
  height: 32px;
  border-radius: 10px;
  font-size: 16px;
}

.brand-copy {
  display: flex;
  flex-direction: column;
  line-height: 1.2;
}

.brand-sub {
  font-size: 11px;
}
</style>
