<script setup lang="ts">
import { computed, reactive, shallowRef } from 'vue'
import { useI18n } from 'vue-i18n'
import { open } from '@tauri-apps/plugin-dialog'
import { FolderOpenOutlined } from '@antdv-next/icons'
import { Alert, Button, Flex, Form, FormItem, Input, Modal, Select, TypographyParagraph } from 'antdv-next'
import { storeToRefs } from 'pinia'
import { CATALOG_FILTERS, reimport } from '../api'
import { useWorkspace } from '../stores/workspace'

const { t } = useI18n()
const store = useWorkspace()
const { dataTab, metadata, showReimport } = storeToRefs(store)
const busy = shallowRef(false)
const err = shallowRef<string | null>(null)
const model = reactive({
  encoding: metadata.value?.encoding ?? '',
  format: metadata.value?.fileFormat ?? '',
  catalogPath: metadata.value?.catalogPath ?? '',
})

const encodingOptions = computed(() => [
  { label: t('reimport.encDefault'), value: '' },
  { label: 'GBK', value: 'GBK' },
  { label: 'GB18030', value: 'GB18030' },
  { label: 'GB2312', value: 'GB2312' },
  { label: 'Latin1 / ISO-8859-1', value: 'LATIN1' },
  { label: 'Windows-1252', value: 'WINDOWS-1252' },
  { label: 'UTF-8', value: 'UTF-8' },
])

const formatOptions = computed(() => [
  { label: t('reimport.fmtDefault'), value: '' },
  { label: t('reimport.fmtSas'), value: 'sas7bdat' },
  { label: t('reimport.fmtXpt'), value: 'xpt' },
  { label: t('reimport.fmtSav'), value: 'sav' },
  { label: t('reimport.fmtPor'), value: 'por' },
  { label: t('reimport.fmtDta'), value: 'dta' },
])

async function pickCatalog() {
  const selected = await open({
    multiple: false,
    filters: CATALOG_FILTERS.map(({ key, extensions }) => ({ name: t(key), extensions })),
  })
  if (typeof selected === 'string') model.catalogPath = selected
}

async function submit() {
  if (!dataTab.value) return
  busy.value = true
  err.value = null
  try {
    await reimport(dataTab.value.tableName, {
      path: dataTab.value.path,
      encoding: model.encoding || undefined,
      format: model.format || undefined,
      catalogPath: model.catalogPath || undefined,
    })
    store.showReimport = false
    await store.refresh()
  } catch (error) {
    err.value = String(error)
  } finally {
    busy.value = false
  }
}
</script>

<template>
  <Modal
    v-model:open="showReimport"
    :title="t('reimport.title')"
    :ok-text="t('reimport.ok')"
    :confirm-loading="busy"
    destroy-on-hidden
    @ok="submit"
  >
    <Form layout="vertical" :model="model">
      <TypographyParagraph type="secondary">
        {{ t('reimport.hint') }}
      </TypographyParagraph>
      <FormItem :label="t('reimport.encoding')" name="encoding">
        <Select v-model:value="model.encoding" :options="encodingOptions" />
      </FormItem>
      <FormItem :label="t('reimport.format')" name="format">
        <Select v-model:value="model.format" :options="formatOptions" />
      </FormItem>
      <FormItem :label="t('reimport.catalog')" name="catalogPath">
        <Flex gap="small">
          <Input v-model:value="model.catalogPath" :placeholder="t('reimport.optional')" />
          <Button @click="pickCatalog">
            <template #icon><FolderOpenOutlined /></template>
            {{ t('reimport.browse') }}
          </Button>
        </Flex>
      </FormItem>
      <Alert v-if="err" type="error" show-icon :title="err" />
    </Form>
  </Modal>
</template>
