<script setup lang="ts">
import { computed } from 'vue'
import { Tooltip, theme } from 'antdv-next'
import type { HeaderMode } from '../../types'
import { headerDisplay } from '../../utils/columnType'
import ColumnTypeIcon from './ColumnTypeIcon.vue'

const props = defineProps<{
  name: string
  label: string | null
  mode: HeaderMode
  storageType: string
  displayFormat: string | null
  isDatetime: boolean
}>()

const { token } = theme.useToken()
const lines = computed(() => headerDisplay(props.mode, props.name, props.label))
</script>

<template>
  <span class="header-title" :class="{ stacked: Boolean(lines.secondary) }">
    <ColumnTypeIcon
      :storage-type="storageType"
      :display-format="displayFormat"
      :is-datetime="isDatetime"
    />
    <Tooltip :mouse-enter-delay="0.25">
      <template #title>
        <div>{{ name }}</div>
        <div v-if="label">{{ label }}</div>
      </template>
      <span class="header-copy">
        <span class="header-line">{{ lines.primary }}</span>
        <span v-if="lines.secondary" class="header-line is-sub">{{ lines.secondary }}</span>
      </span>
    </Tooltip>
  </span>
</template>

<style scoped>
.header-title {
  display: flex;
  flex: 1;
  gap: 6px;
  align-items: center;
  min-width: 0;
}

.header-title.stacked {
  align-items: center;
}

.header-title > :deep(*) {
  min-width: 0;
}

.header-title > :deep(*:last-child) {
  flex: 1;
}

.header-copy {
  display: flex;
  flex: 1;
  flex-direction: column;
  min-width: 0;
  overflow: hidden;
  line-height: 1.2;
}

.header-line {
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.header-line.is-sub {
  font-size: 11px;
  font-weight: 400;
  color: v-bind('token.colorTextSecondary');
}
</style>
