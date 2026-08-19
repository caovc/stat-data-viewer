<script setup lang="ts">
import { shallowRef, useTemplateRef } from 'vue'
import { Tooltip } from 'antdv-next'

defineProps<{
  text: string
}>()

const el = useTemplateRef<HTMLElement>('el')
const tip = shallowRef<string>()

function onEnter() {
  const node = el.value
  const text = node?.textContent ?? ''
  tip.value = node && text && node.scrollWidth > node.clientWidth + 1 ? text : undefined
}
</script>

<template>
  <Tooltip
    :title="tip"
    :mouse-enter-delay="0.25"
    :styles="{ container: { maxWidth: '360px', wordBreak: 'break-word' } }"
  >
    <span ref="el" class="cell-text" @mouseenter="onEnter">{{ text }}</span>
  </Tooltip>
</template>

<style scoped>
:deep(.ant-tooltip-disabled-compatible-wrapper) {
  display: block;
  min-width: 0;
  max-width: 100%;
}

.cell-text {
  display: block;
  min-width: 0;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}
</style>
