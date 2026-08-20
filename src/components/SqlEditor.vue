<script setup lang="ts">
import { computed, onMounted, useTemplateRef } from 'vue'
import { useI18n } from 'vue-i18n'
import { PlayCircleOutlined } from '@antdv-next/icons'
import { Button, Flex, Tag, TypographyText, theme } from 'antdv-next'
import { storeToRefs } from 'pinia'
import { useMonacoSqlEditor } from '../composables/useMonacoSqlEditor'
import { usePreferences } from '../stores/preferences'
import { useWorkspace } from '../stores/workspace'

const { t } = useI18n()
const { token } = theme.useToken()
const store = useWorkspace()
const prefs = usePreferences()
const { sqlDraft, metadata, sqlCatalog, showSql } = storeToRefs(store)
const { resolvedTheme } = storeToRefs(prefs)
const host = useTemplateRef<HTMLDivElement>('sqlMonaco')

useMonacoSqlEditor({
  container: host,
  value: sqlDraft,
  catalog: sqlCatalog,
  visible: showSql,
  theme: computed(() =>
    resolvedTheme.value === 'dark' ? 'stat-data-sql-dark' : 'stat-data-sql-light',
  ),
  placeholder: computed(() => t('sql.placeholder')),
  onRun: () => {
    void store.runActiveSql()
  },
})

onMounted(() => {
  void store.hydrateSqlCatalog()
})
</script>

<template>
  <section class="sql-panel" :data-theme="resolvedTheme">
    <Flex class="sql-head" align="center" justify="space-between">
      <Flex align="center" gap="small">
        <TypographyText strong>{{ t('sql.title') }}</TypographyText>
        <Tag v-if="metadata" bordered>{{ metadata.tableName }}</Tag>
      </Flex>
      <Flex gap="small">
        <Button type="primary" :title="t('sql.runHint')" @click="store.runActiveSql()">
          <template #icon><PlayCircleOutlined /></template>
          {{ t('sql.run') }}
        </Button>
        <Button @click="store.showSql = false">{{ t('sql.hide') }}</Button>
      </Flex>
    </Flex>
    <div ref="sqlMonaco" class="sql-monaco" />
  </section>
</template>

<style scoped>
.sql-panel {
  display: flex;
  flex-direction: column;
  height: 100%;
  background: v-bind('token.colorBgContainer');
}

.sql-head {
  padding: 8px 12px;
  border-bottom: 1px solid v-bind('token.colorBorderSecondary');
}

.sql-monaco {
  flex: 1;
  min-height: 0;
}

.sql-monaco :deep(.monaco-editor),
.sql-monaco :deep(.overflow-guard) {
  border-radius: 0;
}

.sql-panel[data-theme='light'] {
  --sql-widget-bg: #ffffff;
  --sql-widget-fg: #0f172a;
  --sql-widget-muted: #64748b;
  --sql-widget-border: #d0d5dd;
  --sql-widget-shadow: 0 10px 28px rgba(15, 23, 42, 0.14);
  --sql-widget-code-bg: #f1f5f9;
}

.sql-panel[data-theme='dark'] {
  --sql-widget-bg: #1d232b;
  --sql-widget-fg: #e2e8f0;
  --sql-widget-muted: #94a3b8;
  --sql-widget-border: #3f4a57;
  --sql-widget-shadow: 0 12px 32px rgba(0, 0, 0, 0.45);
  --sql-widget-code-bg: #0f141a;
}

.sql-monaco :deep(.suggest-widget),
.sql-monaco :deep(.monaco-hover),
.sql-monaco :deep(.parameter-hints-widget) {
  border: 1px solid var(--sql-widget-border) !important;
  border-radius: 8px !important;
  box-shadow: var(--sql-widget-shadow) !important;
  background: var(--sql-widget-bg) !important;
  color: var(--sql-widget-fg) !important;
}

.sql-monaco :deep(.suggest-widget .monaco-list .monaco-list-row) {
  color: var(--sql-widget-fg);
}

.sql-monaco :deep(.suggest-widget .monaco-icon-label-description-container .label-description),
.sql-monaco :deep(.suggest-widget .monaco-icon-label-description-container .label-details) {
  color: var(--sql-widget-muted);
  opacity: 1;
}

.sql-monaco :deep(.monaco-hover .hover-contents) {
  padding: 8px 10px;
  max-width: 440px;
  color: var(--sql-widget-fg);
}

.sql-monaco :deep(.monaco-hover p) {
  margin: 0.35em 0;
  color: var(--sql-widget-fg);
}

.sql-monaco :deep(.monaco-hover code) {
  display: inline-block;
  max-width: 100%;
  padding: 1px 5px;
  border-radius: 4px;
  background: var(--sql-widget-code-bg);
  color: var(--sql-widget-fg);
  font-size: 11px;
  line-height: 1.45;
  white-space: pre-wrap;
  word-break: break-all;
}
</style>
