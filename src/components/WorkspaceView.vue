<script setup lang="ts">
import { storeToRefs } from 'pinia'
import { Spin, Splitter, SplitterPanel, theme } from 'antdv-next'
import DataGrid from './DataGrid.vue'
import EmptyState from './EmptyState.vue'
import MetadataPanel from './MetadataPanel.vue'
import SqlEditor from './SqlEditor.vue'
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
  <Splitter class="workspace" :styles="panelLock">
    <SplitterPanel :default-size="280" :min="200" :max="460" collapsible>
      <MetadataPanel />
    </SplitterPanel>
    <SplitterPanel>
      <Splitter v-if="showSql" orientation="vertical" class="workspace-main" :styles="panelLock">
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
    </SplitterPanel>
  </Splitter>
</template>

<style scoped>
.workspace,
.workspace-main,
.workspace-spin {
  width: 100%;
  height: 100%;
  min-width: 0;
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
