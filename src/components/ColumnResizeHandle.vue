<script setup lang="ts">
import { theme } from 'antdv-next'

defineProps<{
  active: boolean
}>()

const emit = defineEmits<{
  start: [event: MouseEvent | TouchEvent]
  reset: []
}>()

const { token } = theme.useToken()
</script>

<template>
  <span
    class="col-resizer"
    :class="{ active }"
    role="separator"
    aria-orientation="vertical"
    aria-label="Resize column"
    @mousedown="emit('start', $event)"
    @touchstart.prevent="emit('start', $event)"
    @dblclick.stop="emit('reset')"
    @click.stop
  />
</template>

<style scoped>
.col-resizer {
  position: absolute;
  top: 0;
  right: -3px;
  z-index: 6;
  width: 8px;
  height: 100%;
  cursor: col-resize;
}

.col-resizer::after {
  content: "";
  position: absolute;
  top: 8px;
  bottom: 8px;
  left: 3px;
  width: 2px;
  border-radius: 1px;
  background: transparent;
}

.col-resizer:hover::after,
.col-resizer.active::after {
  background: v-bind('token.colorPrimary');
}
</style>
