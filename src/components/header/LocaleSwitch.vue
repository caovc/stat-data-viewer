<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { TranslationOutlined } from '@antdv-next/icons'
import { Button, Dropdown, Tooltip } from 'antdv-next'
import { storeToRefs } from 'pinia'
import { usePreferences } from '../../stores/preferences'
import type { AppLocale } from '../../preferences'

const { t } = useI18n()
const prefs = usePreferences()
const { locale } = storeToRefs(prefs)

const items = computed(() => [
  { key: 'zh-CN', label: '简体中文' },
  { key: 'en-US', label: 'English' },
])

function onClick({ key }: { key: string }) {
  prefs.setLocale(key as AppLocale)
}
</script>

<template>
  <Tooltip :title="t('prefs.language')">
    <Dropdown
      :menu="{ items, selectedKeys: [locale] }"
      :trigger="['click']"
      placement="bottomRight"
      @menu-click="onClick"
    >
      <Button>
        <template #icon><TranslationOutlined /></template>
      </Button>
    </Dropdown>
  </Tooltip>
</template>
