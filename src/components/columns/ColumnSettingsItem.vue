<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import {
  HolderOutlined,
  VerticalLeftOutlined,
  VerticalRightOutlined,
} from '@antdv-next/icons'
import { Button, Checkbox, Flex, Tooltip, TypographyText, theme } from 'antdv-next'
import type { ColumnPin } from '../../types'
import ColumnTypeIcon from './ColumnTypeIcon.vue'

const props = defineProps<{
  name: string
  label: string | null
  visible: boolean
  pin: ColumnPin
  dragging: boolean
  dragOver: boolean
  storageType: string
  displayFormat: string | null
  isDatetime: boolean
}>()

const emit = defineEmits<{
  toggle: [name: string, visible: boolean]
  pin: [name: string, pin: ColumnPin]
  reorderPointerDown: [event: PointerEvent]
}>()

const { t } = useI18n()
const { token } = theme.useToken()

function onToggle() {
  emit('toggle', props.name, !props.visible)
}

function togglePin(next: Exclude<ColumnPin, null>) {
  emit('pin', props.name, props.pin === next ? null : next)
}

</script>

<template>
  <div
    class="column-row"
    :class="{ dragging, 'drag-over': dragOver, hidden: !visible }"
    :data-reorder-id="name"
  >
    <button
      class="drag-handle"
      type="button"
      :aria-label="t('columns.drag')"
      @pointerdown="emit('reorderPointerDown', $event)"
    >
      <HolderOutlined />
    </button>
    <Checkbox :checked="visible" @change="onToggle" />
    <ColumnTypeIcon
      :storage-type="storageType"
      :display-format="displayFormat"
      :is-datetime="isDatetime"
    />
    <div class="column-copy">
      <TypographyText
        :ellipsis="{ tooltip: name }"
        :disabled="!visible"
        class="column-name"
      >
        {{ name }}
      </TypographyText>
      <TypographyText
        v-if="label"
        type="secondary"
        :ellipsis="{ tooltip: label }"
        class="column-label"
      >
        {{ label }}
      </TypographyText>
    </div>
    <Flex class="pin-actions" :gap="4">
      <Tooltip :title="t('columns.pinLeft')">
        <Button
          size="small"
          type="text"
          :color="pin === 'start' ? 'primary' : 'default'"
          :variant="pin === 'start' ? 'filled' : 'text'"
          @click="togglePin('start')"
        >
          <template #icon><VerticalRightOutlined /></template>
        </Button>
      </Tooltip>
      <Tooltip :title="t('columns.pinRight')">
        <Button
          size="small"
          type="text"
          :color="pin === 'end' ? 'primary' : 'default'"
          :variant="pin === 'end' ? 'filled' : 'text'"
          @click="togglePin('end')"
        >
          <template #icon><VerticalLeftOutlined /></template>
        </Button>
      </Tooltip>
    </Flex>
  </div>
</template>

<style scoped>
.column-row {
  display: grid;
  grid-template-columns: 28px 22px 16px minmax(0, 1fr) auto;
  gap: 8px;
  align-items: center;
  min-height: 40px;
  padding: 6px 8px;
  border-radius: 8px;
  border: 1px solid transparent;
}

.column-row.drag-over {
  border-color: v-bind('token.colorPrimary');
  background: v-bind('token.colorPrimaryBg');
}

.column-row.dragging {
  opacity: 0.45;
}

.column-row.hidden .column-copy {
  opacity: 0.55;
}

.drag-handle {
  display: inline-flex;
  align-items: center;
  justify-content: center;
  width: 28px;
  height: 28px;
  padding: 0;
  border: 0;
  color: v-bind('token.colorTextTertiary');
  background: transparent;
  touch-action: none;
  cursor: grab;
}

.drag-handle:active {
  cursor: grabbing;
}

.column-copy {
  display: flex;
  flex-direction: column;
  gap: 1px;
  min-width: 0;
  overflow: hidden;
  line-height: 1.25;
}

.column-name,
.column-label {
  display: block;
  max-width: 100%;
}

.column-label {
  font-size: 12px;
}

.pin-actions :deep(.ant-btn) {
  min-width: 28px;
}
</style>
