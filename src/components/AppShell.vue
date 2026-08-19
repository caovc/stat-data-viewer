<script setup lang="ts">
import { onMounted, onUnmounted } from 'vue'
import { getCurrentWebview } from '@tauri-apps/api/webview'
import { storeToRefs } from 'pinia'
import { Alert, Layout, LayoutContent, LayoutFooter, LayoutHeader, theme } from 'antdv-next'
import AppHeader from './AppHeader.vue'
import ColumnsDrawer from './ColumnsDrawer.vue'
import MetadataPanel from './MetadataPanel.vue'
import SortFilterDrawer from './query/SortFilterDrawer.vue'
import ExportDialog from './ExportDialog.vue'
import ReimportDialog from './ReimportDialog.vue'
import StatusBar from './StatusBar.vue'
import WorkspaceTabs from './WorkspaceTabs.vue'
import WorkspaceView from './WorkspaceView.vue'
import { useAppMenu } from '../composables/useAppMenu'
import { useWorkspaceActions } from '../composables/useWorkspaceActions'

const { token } = theme.useToken()
const { store, openFiles } = useWorkspaceActions()
const { error, dragging, showReimport, showExport } = storeToRefs(store)

useAppMenu(openFiles)

function onContextMenu(event: MouseEvent) {
  event.preventDefault()
}

async function onKey(event: KeyboardEvent) {
  const meta = event.metaKey || event.ctrlKey
  if (meta && event.key.toLowerCase() === 'o') {
    event.preventDefault()
    await openFiles()
  }
  if (meta && event.key.toLowerCase() === 'e') {
    event.preventDefault()
    store.showExport = true
  }
}

onMounted(async () => {
  window.addEventListener('contextmenu', onContextMenu)
  window.addEventListener('keydown', onKey)
  await store.bindEvents()
  const unlisten = await getCurrentWebview().onDragDropEvent(async (event) => {
    if (event.payload.type === 'over') store.dragging = true
    if (event.payload.type === 'leave') store.dragging = false
    if (event.payload.type === 'drop') {
      store.dragging = false
      for (const path of event.payload.paths) await store.openPath(path)
    }
  })
  onUnmounted(() => {
    window.removeEventListener('contextmenu', onContextMenu)
    window.removeEventListener('keydown', onKey)
    unlisten()
  })
})
</script>

<template>
  <Layout class="app-layout">
    <LayoutHeader class="app-header">
      <AppHeader />
    </LayoutHeader>
    <WorkspaceTabs />
    <Alert
      v-if="error"
      type="error"
      banner
      show-icon
      closable
      :title="error"
      @close="store.error = null"
    />
    <LayoutContent class="app-content">
      <WorkspaceView />
    </LayoutContent>
    <LayoutFooter class="app-footer">
      <StatusBar />
    </LayoutFooter>
    <MetadataPanel />
    <ColumnsDrawer />
    <SortFilterDrawer />
    <ReimportDialog v-if="showReimport" />
    <ExportDialog v-if="showExport" />
    <div
      v-if="dragging"
      class="drop-overlay"
      :style="{
        borderColor: token.colorPrimary,
        background: token.colorPrimaryBg,
      }"
    />
  </Layout>
</template>

<style scoped>
.app-layout {
  height: 100%;
}

.app-header {
  display: flex;
  align-items: center;
  border-bottom: 1px solid v-bind('token.colorBorderSecondary');
  line-height: 1;
}

.app-content {
  min-width: 0;
  min-height: 0;
  overflow: hidden;
  padding: 0;
}

.app-footer {
  display: flex;
  align-items: center;
  min-height: 48px;
  border-top: 1px solid v-bind('token.colorBorderSecondary');
}

.drop-overlay {
  position: fixed;
  inset: 12px;
  z-index: 20;
  border: 2px dashed;
  border-radius: 16px;
  pointer-events: none;
}
</style>
