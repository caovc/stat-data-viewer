import { watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { Menu, MenuItem, PredefinedMenuItem, Submenu } from '@tauri-apps/api/menu'

/** Label used by Tauri's default File submenu (not localized). */
const DEFAULT_FILE_LABEL = 'File'

export function useAppMenu(openFiles: () => void | Promise<void>) {
  const { t, locale } = useI18n()

  async function findFileSubmenu(menu: Menu) {
    for (const item of await menu.items()) {
      if (item instanceof Submenu && (await item.text()) === DEFAULT_FILE_LABEL) {
        return item
      }
    }
    return null
  }

  async function install() {
    const menu = await Menu.default()
    const openItem = await MenuItem.new({
      id: 'open-file',
      text: t('menu.open'),
      accelerator: 'CmdOrCtrl+O',
      action: () => {
        void openFiles()
      },
    })

    const file = await findFileSubmenu(menu)
    if (file) {
      await file.prepend([
        openItem,
        await PredefinedMenuItem.new({ item: 'Separator' }),
      ])
      await file.setText(t('menu.file'))
    } else {
      await menu.insert(
        await Submenu.new({
          text: t('menu.file'),
          items: [openItem],
        }),
        0,
      )
    }

    await menu.setAsAppMenu()
  }

  watch(
    locale,
    () => {
      void install().catch(() => {
        // Vite-only preview has no native menu.
      })
    },
    { immediate: true },
  )
}
