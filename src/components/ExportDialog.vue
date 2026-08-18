<script setup lang="ts">
import { computed, reactive, shallowRef } from 'vue'
import { useI18n } from 'vue-i18n'
import { save } from '@tauri-apps/plugin-dialog'
import { Alert, Form, FormItem, Modal, Select, TypographyParagraph } from 'antdv-next'
import { storeToRefs } from 'pinia'
import { exportResult } from '../api'
import { useWorkspace } from '../stores/workspace'

const { t } = useI18n()
const store = useWorkspace()
const { dataTab, sqlDraft, active, showExport } = storeToRefs(store)
const busy = shallowRef(false)
const err = shallowRef<string | null>(null)
const model = reactive({ format: 'csv' })

const formatOptions = computed(() => [
  { label: t('export.csv'), value: 'csv' },
  { label: t('export.parquet'), value: 'parquet' },
  { label: t('export.excel'), value: 'excel' },
])

async function submit() {
  const ext = model.format === 'excel' ? 'xlsx' : model.format
  const dest = await save({
    defaultPath: `${dataTab.value?.tableName ?? 'query'}.${ext}`,
    filters: [
      { name: t('export.csv'), extensions: ['csv'] },
      { name: t('export.parquet'), extensions: ['parquet'] },
      { name: t('export.excel'), extensions: ['xlsx'] },
    ],
  })
  if (!dest) return
  busy.value = true
  err.value = null
  try {
    const useSql = active.value?.kind === 'sql'
    await exportResult({
      path: dest,
      format: model.format,
      table: useSql ? undefined : dataTab.value?.tableName,
      sql: useSql ? sqlDraft.value : undefined,
    })
    store.showExport = false
  } catch (error) {
    err.value = String(error)
  } finally {
    busy.value = false
  }
}
</script>

<template>
  <Modal
    v-model:open="showExport"
    :title="t('export.title')"
    :ok-text="t('export.save')"
    :confirm-loading="busy"
    destroy-on-hidden
    @ok="submit"
  >
    <Form layout="vertical" :model="model">
      <FormItem :label="t('export.format')" name="format">
        <Select v-model:value="model.format" :options="formatOptions" />
      </FormItem>
      <TypographyParagraph type="secondary">
        {{ t('export.hint') }}
      </TypographyParagraph>
      <Alert v-if="err" type="error" show-icon :title="err" />
    </Form>
  </Modal>
</template>
