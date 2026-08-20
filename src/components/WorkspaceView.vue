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

const { token } = theme.useToken()
const { store, openFiles } = useWorkspaceActions()
const { tabs, showSql, loading } = storeToRefs(store)
const sqlOpened = ref(false)

watch(
  showSql,
  (open) => {
    if (!open) return
    sqlOpened.value = true
    preloadSqlEditor()
  },
  { immediate: true },
)
</script>

<template>
  <div class="workspace">
    <div class="workspace-main">
      <ViewToolbar v-if="tabs.length > 0" />
      <div class="workspace-body">
        <Spin :spinning="loading" class="workspace-spin">
          <EmptyState v-if="tabs.length === 0" @open="openFiles" />
          <DataGrid v-else />
        </Spin>
        <div
          v-if="sqlOpened"
          v-show="showSql"
          class="sql-pane"
          :style="{ borderTop: `1px solid ${token.colorBorderSecondary}` }"
        >
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
  flex: none;
  height: 220px;
  min-height: 140px;
  background: v-bind('token.colorBgContainer');
}
</style>
