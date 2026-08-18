<script setup lang="ts">
import { computed, h } from 'vue'
import { useI18n } from 'vue-i18n'
import { DesktopOutlined, MoonOutlined, SunOutlined } from '@antdv-next/icons'
import { Button, Dropdown, Tooltip } from 'antdv-next'
import { storeToRefs } from 'pinia'
import { usePreferences } from '../../stores/preferences'
import type { ThemeMode } from '../../preferences'

const { t } = useI18n()
const prefs = usePreferences()
const { themeMode, resolvedTheme } = storeToRefs(prefs)

const items = computed(() => [
  { key: 'light', label: t('prefs.themeLight'), icon: h(SunOutlined) },
  { key: 'dark', label: t('prefs.themeDark'), icon: h(MoonOutlined) },
  { key: 'system', label: t('prefs.themeSystem'), icon: h(DesktopOutlined) },
])

function onClick({ key }: { key: string }) {
  prefs.setThemeMode(key as ThemeMode)
}
</script>

<template>
  <Tooltip :title="t('prefs.theme')">
    <Dropdown
      :menu="{ items, selectedKeys: [themeMode] }"
      :trigger="['click']"
      placement="bottomRight"
      @menu-click="onClick"
    >
      <Button>
        <template #icon>
          <MoonOutlined v-if="resolvedTheme === 'dark'" />
          <SunOutlined v-else />
        </template>
      </Button>
    </Dropdown>
  </Tooltip>
</template>
