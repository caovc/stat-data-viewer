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
const { sqlDraft, metadata, sqlCatalog } = storeToRefs(store)
const { resolvedTheme } = storeToRefs(prefs)
const host = useTemplateRef<HTMLDivElement>('sqlMonaco')

useMonacoSqlEditor({
  container: host,
  value: sqlDraft,
  catalog: sqlCatalog,
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
  <section class="sql-panel">
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
</style>
