<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { TypographyText, theme } from 'antdv-next'
import ColumnSettingsItem from './ColumnSettingsItem.vue'
import { usePointerReorder } from '../../composables/usePointerReorder'
import type { ColumnPin, ColumnSetting } from '../../types'

const props = defineProps<{
  items: ColumnSetting[]
}>()

const emit = defineEmits<{
  toggle: [name: string, visible: boolean]
  pin: [name: string, pin: ColumnPin]
  reorder: [from: string, to: string]
}>()

const { t } = useI18n()
const { token } = theme.useToken()

const groups = computed(() => [
  { key: 'start', title: t('columns.pinnedLeft'), items: props.items.filter((item) => item.pin === 'start') },
  { key: 'middle', title: t('columns.group'), items: props.items.filter((item) => item.pin === null) },
  { key: 'end', title: t('columns.pinnedRight'), items: props.items.filter((item) => item.pin === 'end') },
])

function pinOfName(name: string) {
  return props.items.find((item) => item.name === name)?.pin ?? null
}

function sameGroup(from: string, to: string) {
  return pinOfName(from) === pinOfName(to)
}

const { dragId: dragName, overId: overName, onHandlePointerDown } = usePointerReorder(
  (from, to) => emit('reorder', from, to),
  sameGroup,
)

function onToggle(name: string, visible: boolean) {
  emit('toggle', name, visible)
}

function onPin(name: string, pin: ColumnPin) {
  emit('pin', name, pin)
}
</script>

<template>
  <div class="column-groups">
    <section v-for="group in groups" v-show="group.items.length" :key="group.key" class="column-group">
      <TypographyText type="secondary" class="group-title">
        {{ group.title }}
      </TypographyText>
      <ColumnSettingsItem
        v-for="item in group.items"
        :key="item.name"
        :name="item.name"
        :label="item.label"
        :visible="item.visible"
        :pin="item.pin"
        :storage-type="item.storageType"
        :display-format="item.displayFormat"
        :is-datetime="item.isDatetime"
        :dragging="dragName === item.name"
        :drag-over="overName === item.name && dragName !== item.name"
        @toggle="onToggle"
        @pin="onPin"
        @reorder-pointer-down="onHandlePointerDown(item.name, $event)"
      />
    </section>
  </div>
</template>

<style scoped>
.column-groups {
  display: flex;
  flex-direction: column;
  gap: 12px;
}

.group-title {
  display: block;
  margin: 0 8px 4px;
  font-size: 12px;
}

.column-group {
  padding: 4px;
  border-radius: 10px;
  background: v-bind('token.colorFillQuaternary');
}
</style>
