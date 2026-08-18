<script setup lang="ts">
import { computed } from 'vue'
import { App as AntdApp, ConfigProvider } from 'antdv-next'
import enUS from 'antdv-next/locale/en_US'
import zhCN from 'antdv-next/locale/zh_CN'
import { storeToRefs } from 'pinia'
import AppShell from './components/AppShell.vue'
import { usePreferences } from './stores/preferences'
import { createAppTheme } from './theme'

const prefs = usePreferences()
const { locale, resolvedTheme } = storeToRefs(prefs)

const antdLocale = computed(() => (locale.value === 'zh-CN' ? zhCN : enUS))
const appTheme = computed(() => createAppTheme(resolvedTheme.value === 'dark'))
</script>

<template>
  <ConfigProvider :theme="appTheme" :locale="antdLocale" component-size="middle">
    <AntdApp class="app-root">
      <AppShell />
    </AntdApp>
  </ConfigProvider>
</template>
