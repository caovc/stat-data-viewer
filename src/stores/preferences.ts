import { computed, onScopeDispose, ref, watch } from 'vue'
import { defineStore } from 'pinia'
import { i18n } from '../i18n'
import {
  applyDocumentChrome,
  detectSystemTheme,
  loadPreferences,
  resolveTheme,
  savePreferences,
  type AppLocale,
  type ThemeMode,
} from '../preferences'

export const usePreferences = defineStore('preferences', () => {
  const stored = loadPreferences()
  const themeMode = ref<ThemeMode>(stored.themeMode)
  const locale = ref<AppLocale>(stored.locale)
  const systemTheme = ref(detectSystemTheme())
  const resolvedTheme = computed(() => resolveTheme(themeMode.value, systemTheme.value))

  const media = window.matchMedia('(prefers-color-scheme: dark)')
  const onSystemTheme = (event: MediaQueryListEvent) => {
    systemTheme.value = event.matches ? 'dark' : 'light'
  }
  media.addEventListener('change', onSystemTheme)
  onScopeDispose(() => media.removeEventListener('change', onSystemTheme))

  watch(
    [themeMode, locale],
    () => savePreferences({ themeMode: themeMode.value, locale: locale.value }),
  )

  watch(
    [resolvedTheme, locale],
    () => {
      applyDocumentChrome(resolvedTheme.value, locale.value)
      i18n.global.locale.value = locale.value
    },
    { immediate: true },
  )

  function setThemeMode(mode: ThemeMode) {
    themeMode.value = mode
  }

  function setLocale(next: AppLocale) {
    locale.value = next
  }

  return {
    themeMode,
    locale,
    systemTheme,
    resolvedTheme,
    setThemeMode,
    setLocale,
  }
})
