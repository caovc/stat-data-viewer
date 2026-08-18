export const LOCALES = ['zh-CN', 'en-US'] as const
export type AppLocale = (typeof LOCALES)[number]

export const THEME_MODES = ['light', 'dark', 'system'] as const
export type ThemeMode = (typeof THEME_MODES)[number]
export type ResolvedTheme = 'light' | 'dark'

export const PREFS_STORAGE_KEY = 'stat-data-viewer:preferences'

export interface StoredPreferences {
  themeMode: ThemeMode
  locale: AppLocale
}

export function isLocale(value: unknown): value is AppLocale {
  return LOCALES.includes(value as AppLocale)
}

export function isThemeMode(value: unknown): value is ThemeMode {
  return THEME_MODES.includes(value as ThemeMode)
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
    if (!raw) return { themeMode: 'system', locale: detectLocale() }
    const parsed = JSON.parse(raw) as Partial<StoredPreferences>
    return {
      themeMode: isThemeMode(parsed.themeMode) ? parsed.themeMode : 'system',
      locale: isLocale(parsed.locale) ? parsed.locale : detectLocale(),
    }
  } catch {
    return { themeMode: 'system', locale: detectLocale() }
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
