<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { useI18n } from 'vue-i18n'
import { PlusOutlined } from '@antdv-next/icons'
import { Button, Drawer, Flex, TypographyParagraph, TypographyText } from 'antdv-next'
import { storeToRefs } from 'pinia'
import FilterGroupEditor from './FilterGroupEditor.vue'
import SortRuleList from './SortRuleList.vue'
import { useWorkspace } from '../../stores/workspace'
import { typeFieldsOf } from '../../utils/columnType'
import {
  cloneFilterTree,
  cloneSorts,
  emptyFilterDraft,
  emptyFilterGroup,
  firstUnusedColumn,
  isEmptyFilterDraft,
  nextRuleId,
  sameFilters,
  sameSorts,
  toFilterGroup,
  toSortSpecs,
  type FilterGroupDraft,
  type QueryColumn,
  type SortDraft,
} from '../../utils/queryRules'

const { t } = useI18n()
const store = useWorkspace()
const { metadata, page, sorts, filters, showQuery, dataTab, activeId } = storeToRefs(store)

const draftSorts = ref<SortDraft[]>([])
const draftFilters = ref<FilterGroupDraft>(emptyFilterDraft())

const columns = computed<QueryColumn[]>(() => {
  const source = metadata.value?.variables ?? page.value?.columns ?? []
  return source.map((item) => ({
    name: item.name,
    label: item.label ?? null,
    ...typeFieldsOf(item),
  }))
})

const appliedSorts = computed(() => toSortSpecs(draftSorts.value))
const appliedFilters = computed(() => toFilterGroup(draftFilters.value))
const dirty = computed(() =>
  !sameSorts(appliedSorts.value, sorts.value) || !sameFilters(appliedFilters.value, filters.value),
)
const canClear = computed(() => draftSorts.value.length > 0 || !isEmptyFilterDraft(draftFilters.value))
const canAddSort = computed(() => firstUnusedColumn(columns.value, draftSorts.value.map((item) => item.column)))

function loadDrafts() {
  draftSorts.value = cloneSorts(sorts.value)
  draftFilters.value = cloneFilterTree(filters.value)
}

watch([showQuery, activeId], ([open]) => {
  if (open) loadDrafts()
})

watch([sorts, filters], () => {
  if (showQuery.value) loadDrafts()
})

function addSort() {
  const col = canAddSort.value
  if (!col) return
  draftSorts.value = [...draftSorts.value, { id: nextRuleId('s'), column: col.name, desc: false }]
}

async function apply() {
  await store.applyQuery(appliedSorts.value, appliedFilters.value)
}

async function clearAll() {
  draftSorts.value = []
  draftFilters.value = emptyFilterDraft()
  await store.applyQuery([], emptyFilterGroup())
}
</script>

<template>
  <Drawer
    v-model:open="showQuery"
    :title="t('query.title')"
    placement="right"
    :size="560"
    destroy-on-hidden
    resizable
  >
    <Flex vertical gap="middle" class="query-panel">
      <section>
        <TypographyText strong>{{ t('query.sort') }}</TypographyText>
        <TypographyParagraph type="secondary" class="section-hint">
          {{ t('query.sortHint') }}
        </TypographyParagraph>
        <TypographyText v-if="draftSorts.length === 0" type="secondary" class="empty-copy">
          {{ t('query.sortEmpty') }}
        </TypographyText>
        <SortRuleList v-else v-model="draftSorts" :columns="columns" />
        <Button class="add-btn" :disabled="!canAddSort" @click="addSort">
          <template #icon><PlusOutlined /></template>
          {{ t('query.addSort') }}
        </Button>
      </section>

      <section>
        <TypographyText strong>{{ t('query.filter') }}</TypographyText>
        <TypographyParagraph type="secondary" class="section-hint">
          {{ t('query.filterHint') }}
        </TypographyParagraph>
        <FilterGroupEditor
          v-model="draftFilters"
          root
          :columns="columns"
          :table="dataTab?.tableName"
        />
      </section>
    </Flex>

    <template #footer>
      <Flex justify="space-between" gap="small">
        <Button :disabled="!canClear" @click="clearAll">{{ t('query.clear') }}</Button>
        <Button type="primary" :disabled="!dirty" @click="apply">{{ t('query.apply') }}</Button>
      </Flex>
    </template>
  </Drawer>
</template>

<style scoped>
.query-panel {
  min-width: 0;
}

.section-hint {
  margin: 4px 0 12px;
}

.empty-copy {
  display: block;
  margin-bottom: 8px;
}

.add-btn {
  width: 100%;
  margin-top: 8px;
}
</style>
