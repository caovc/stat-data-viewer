import { createI18n } from 'vue-i18n'
import { loadPreferences } from '../preferences'
import enUS from './locales/en-US'
import zhCN from './locales/zh-CN'

export type MessageSchema = typeof enUS

export const i18n = createI18n({
  legacy: false,
  locale: loadPreferences().locale,
  fallbackLocale: 'en-US',
  messages: {
    'en-US': enUS,
    'zh-CN': zhCN,
  },
})
