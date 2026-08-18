import { theme, type ThemeConfig } from 'antdv-next'

const sharedToken = {
  colorPrimary: '#0f766e',
  colorInfo: '#0f766e',
  colorSuccess: '#15803d',
  colorWarning: '#b45309',
  colorError: '#b91c1c',
  colorLink: '#0f766e',
  borderRadius: 8,
  fontFamily:
    '-apple-system, BlinkMacSystemFont, "Segoe UI Variable Text", "Segoe UI", "PingFang SC", "Hiragino Sans GB", "Microsoft YaHei", sans-serif',
} as const

const lightSurfaces = {
  colorBgLayout: '#eef1f4',
  colorBgContainer: '#ffffff',
  colorBgElevated: '#ffffff',
} as const

const darkSurfaces = {
  colorBgLayout: '#101418',
  colorBgContainer: '#171c22',
  colorBgElevated: '#1d232b',
} as const

export function createAppTheme(isDark: boolean): ThemeConfig {
  return {
    algorithm: isDark
      ? [theme.darkAlgorithm, theme.compactAlgorithm]
      : theme.compactAlgorithm,
    token: {
      ...sharedToken,
      ...(isDark ? darkSurfaces : lightSurfaces),
    },
    components: {
      Layout: {
        headerBg: isDark ? darkSurfaces.colorBgContainer : lightSurfaces.colorBgContainer,
        headerHeight: 56,
        headerPadding: '0 16px',
        footerBg: isDark ? darkSurfaces.colorBgContainer : lightSurfaces.colorBgContainer,
        footerPadding: '0 16px',
        bodyBg: isDark ? darkSurfaces.colorBgLayout : lightSurfaces.colorBgLayout,
      },
      Tabs: {
        cardBg: isDark ? '#12171c' : '#f7f8fa',
        horizontalMargin: '0',
      },
      Table: {
        headerBg: isDark ? '#1b2128' : '#f4f6f8',
        cellPaddingBlockSM: 6,
        cellPaddingInlineSM: 10,
      },
    },
  }
}
