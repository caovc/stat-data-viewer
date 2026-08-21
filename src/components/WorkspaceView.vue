<script setup lang="ts">
import { ref, watch } from 'vue'
import { storeToRefs } from 'pinia'
import { Spin, theme } from 'antdv-next'
import DataGrid from './DataGrid.vue'
import EmptyState from './EmptyState.vue'
import SqlEditor from './SqlEditor.vue'
import ViewToolbar from './ViewToolbar.vue'
import { useWorkspaceActions } from '../composables/useWorkspaceActions'
import { preloadSqlEditor } from '../sql/registerDuckdb'

const SQL_MIN = 140
const SQL_DEFAULT = 220

const { token } = theme.useToken()
const { store, openFiles } = useWorkspaceActions()
const { tabs, showSql, loading } = storeToRefs(store)
const sqlOpened = ref(false)
const sqlHeight = ref(SQL_DEFAULT)
const resizing = ref(false)
const body = ref<HTMLElement | null>(null)

watch(
  showSql,
  (open) => {
    if (!open) return
    sqlOpened.value = true
    preloadSqlEditor()
  },
  { immediate: true },
)

function onResizePointerDown(event: PointerEvent) {
  if (event.button !== 0) return
  const handle = event.currentTarget as HTMLElement
  const startY = event.clientY
  const startHeight = sqlHeight.value
  const maxHeight = Math.max(SQL_MIN, (body.value?.clientHeight ?? 480) - 120)
  handle.setPointerCapture(event.pointerId)
  resizing.value = true

  function onMove(move: PointerEvent) {
    sqlHeight.value = Math.min(maxHeight, Math.max(SQL_MIN, startHeight + (startY - move.clientY)))
  }

  function onUp() {
    handle.removeEventListener('pointermove', onMove)
    handle.removeEventListener('pointerup', onUp)
    handle.removeEventListener('pointercancel', onUp)
    resizing.value = false
    try {
      handle.releasePointerCapture(event.pointerId)
    } catch {
      // capture may already be released
    }
  }

  handle.addEventListener('pointermove', onMove)
  handle.addEventListener('pointerup', onUp)
  handle.addEventListener('pointercancel', onUp)
}
</script>

<template>
  <div class="workspace">
    <div class="workspace-main">
      <ViewToolbar v-if="tabs.length > 0" />
      <div ref="body" class="workspace-body">
        <Spin :spinning="loading" class="workspace-spin">
          <EmptyState v-if="tabs.length === 0" @open="openFiles" />
          <DataGrid v-else />
        </Spin>
        <div
          v-if="sqlOpened"
          v-show="showSql"
          class="sql-pane"
          :class="{ resizing }"
          :style="{
            height: `${sqlHeight}px`,
            borderTop: `1px solid ${token.colorBorderSecondary}`,
          }"
        >
          <div
            class="sql-resize"
            role="separator"
            aria-orientation="horizontal"
            :aria-valuenow="sqlHeight"
            :aria-valuemin="SQL_MIN"
            @pointerdown="onResizePointerDown"
          />
          <SqlEditor />
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped>
.workspace,
.workspace-main {
  width: 100%;
  height: 100%;
  min-width: 0;
}

.workspace-main {
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.workspace-body,
.workspace-spin {
  flex: 1;
  width: 100%;
  min-width: 0;
  min-height: 0;
}

.workspace-body {
  display: flex;
  flex-direction: column;
  overflow: hidden;
}

.workspace-spin {
  display: flex;
  overflow: hidden;
}

.workspace-spin :deep(.ant-spin-container) {
  display: flex;
  flex: 1;
  flex-direction: column;
  width: 100%;
  height: 100%;
  min-width: 0;
  min-height: 0;
  overflow: hidden;
}

.sql-pane {
  position: relative;
  display: flex;
  flex: none;
  flex-direction: column;
  min-height: 140px;
  background: v-bind('token.colorBgContainer');
}

.sql-pane.resizing {
  user-select: none;
}

.sql-resize {
  position: absolute;
  top: -4px;
  right: 0;
  left: 0;
  z-index: 2;
  height: 8px;
  cursor: ns-resize;
}

.sql-resize:hover,
.sql-pane.resizing .sql-resize {
  background: v-bind('token.colorPrimary');
  opacity: 0.35;
}
</style>
