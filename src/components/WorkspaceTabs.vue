<script setup lang="ts">
import { computed, h } from 'vue'
import { useI18n } from 'vue-i18n'
import { CodeOutlined, DatabaseOutlined } from '@antdv-next/icons'
import { Button, Tabs, theme } from 'antdv-next'
import { storeToRefs } from 'pinia'
import { useWorkspaceActions } from '../composables/useWorkspaceActions'

const { t } = useI18n()
const { token } = theme.useToken()
const { store, openFiles, addSql } = useWorkspaceActions()
const { tabs, activeId } = storeToRefs(store)

const tabHeads = computed(() =>
  tabs.value.map((tab) => ({
    id: tab.id,
    kind: tab.kind,
    title: tab.title,
    path: tab.kind === 'data' ? tab.path : '',
  })),
)

const items = computed(() =>
  tabHeads.value.map((tab) => {
    const text = tab.kind === 'sql'
      ? t(tab.title === 'SQL result' ? 'tabs.sqlResult' : 'tabs.sql')
      : tab.title
    return {
      key: tab.id,
      label: tab.kind === 'data'
        ? h('span', { class: 'tab-file-name', title: tab.path }, text)
        : text,
      icon: h(tab.kind === 'sql' ? CodeOutlined : DatabaseOutlined),
      closable: true,
    }
  }),
)

function onEdit(targetKey: string | MouseEvent | KeyboardEvent, action: 'add' | 'remove') {
  if (action === 'add') {
    void openFiles()
    return
  }
  if (typeof targetKey === 'string') void store.closeTab(targetKey)
}
</script>

<template>
  <div
    class="tabs-wrap"
    :style="{
      background: token.colorBgContainer,
      borderBottom: `1px solid ${token.colorBorderSecondary}`,
    }"
  >
    <Tabs
      size="small"
      type="editable-card"
      :active-key="activeId ?? undefined"
      :items="items"
      :hide-add="false"
      @change="store.activate"
      @edit="onEdit"
    >
      <template #rightExtra>
        <Button type="text" size="small" @click="addSql">{{ t('tabs.newSql') }}</Button>
      </template>
    </Tabs>
  </div>
</template>

<style scoped>
.tabs-wrap {
  padding: 0 12px;
}

.tabs-wrap :deep(.ant-tabs-nav) {
  margin: 0;
}

.tabs-wrap :deep(.ant-tabs-nav::before) {
  border: 0;
}

.tabs-wrap :deep(.tab-file-name) {
  display: inline-block;
  max-width: 200px;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
  vertical-align: bottom;
}
</style>
