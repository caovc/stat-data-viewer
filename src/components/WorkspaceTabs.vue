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

const items = computed(() =>
  tabs.value.map((tab) => ({
    key: tab.id,
    label: tab.kind === 'sql'
      ? t(tab.title === 'SQL result' ? 'tabs.sqlResult' : 'tabs.sql')
      : tab.title,
    icon: h(tab.kind === 'sql' ? CodeOutlined : DatabaseOutlined),
    closable: true,
  })),
)

function onEdit(targetKey: string | MouseEvent | KeyboardEvent, action: 'add' | 'remove') {
  if (action === 'add') {
    void openFiles()
    return
  }
  if (typeof targetKey === 'string') store.closeTab(targetKey)
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
</style>
