import { SCROLL_MODES, type ScrollMode } from './types'

export const LOCALES = ['zh-CN', 'en-US'] as const
export type AppLocale = (typeof LOCALES)[number]

export const THEME_MODES = ['light', 'dark', 'system'] as const
export type ThemeMode = (typeof THEME_MODES)[number]
export type ResolvedTheme = 'light' | 'dark'

export const PREFS_STORAGE_KEY = 'stat-data-viewer:preferences'

export interface StoredPreferences {
  themeMode: ThemeMode
  locale: AppLocale
  scrollMode: ScrollMode
}

export function isLocale(value: unknown): value is AppLocale {
  return LOCALES.includes(value as AppLocale)
}

export function isThemeMode(value: unknown): value is ThemeMode {
  return THEME_MODES.includes(value as ThemeMode)
}

export function isScrollMode(value: unknown): value is ScrollMode {
  return SCROLL_MODES.includes(value as ScrollMode)
}

export function detectLocale(): AppLocale {
  return navigator.language.toLowerCase().startsWith('zh') ? 'zh-CN' : 'en-US'
}

export function detectSystemTheme(): ResolvedTheme {
  return window.matchMedia('(prefers-color-scheme: dark)').matches ? 'dark' : 'light'
}

export function loadPreferences(): StoredPreferences {
  try {
    const raw = localStorage.getItem(PREFS_STORAGE_KEY)
    if (!raw) return { themeMode: 'system', locale: detectLocale(), scrollMode: 'page' }
    const parsed = JSON.parse(raw) as Partial<StoredPreferences>
    return {
      themeMode: isThemeMode(parsed.themeMode) ? parsed.themeMode : 'system',
      locale: isLocale(parsed.locale) ? parsed.locale : detectLocale(),
      scrollMode: isScrollMode(parsed.scrollMode) ? parsed.scrollMode : 'page',
    }
  } catch {
    return { themeMode: 'system', locale: detectLocale(), scrollMode: 'page' }
  }
}

export function savePreferences(prefs: StoredPreferences) {
  localStorage.setItem(PREFS_STORAGE_KEY, JSON.stringify(prefs))
}

export function resolveTheme(mode: ThemeMode, system: ResolvedTheme): ResolvedTheme {
  return mode === 'system' ? system : mode
}

export function applyDocumentChrome(theme: ResolvedTheme, locale: AppLocale) {
  const root = document.documentElement
  root.dataset.theme = theme
  root.style.colorScheme = theme
  root.lang = locale
}
