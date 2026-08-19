<script setup lang="ts">
import { storeToRefs } from 'pinia'
import { Spin, Splitter, SplitterPanel, theme } from 'antdv-next'
import DataGrid from './DataGrid.vue'
import EmptyState from './EmptyState.vue'
import SqlEditor from './SqlEditor.vue'
import ViewToolbar from './ViewToolbar.vue'
import { useWorkspaceActions } from '../composables/useWorkspaceActions'

const { token } = theme.useToken()
const { store, openFiles } = useWorkspaceActions()
const { tabs, showSql, loading } = storeToRefs(store)

const panelLock = {
  panel: {
    overflow: 'hidden',
    minWidth: 0,
    minHeight: 0,
  },
} as const
</script>

<template>
  <div class="workspace">
    <div class="workspace-main">
      <ViewToolbar v-if="tabs.length > 0" />
      <Splitter v-if="showSql" orientation="vertical" class="workspace-body" :styles="panelLock">
        <SplitterPanel>
          <Spin :spinning="loading" class="workspace-spin">
            <EmptyState v-if="tabs.length === 0" @open="openFiles" />
            <DataGrid v-else />
          </Spin>
        </SplitterPanel>
        <SplitterPanel :default-size="220" :min="140" collapsible>
          <SqlEditor />
        </SplitterPanel>
      </Splitter>
      <Spin v-else :spinning="loading" class="workspace-spin">
        <EmptyState v-if="tabs.length === 0" @open="openFiles" />
        <DataGrid v-else />
      </Spin>
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

.workspace :deep(.ant-splitter-panel) {
  background: v-bind('token.colorBgContainer');
}
</style>
