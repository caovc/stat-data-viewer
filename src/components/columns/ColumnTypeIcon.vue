<script setup lang="ts">
import { computed } from 'vue'
import { useI18n } from 'vue-i18n'
import { Tooltip } from 'antdv-next'
import { columnTypeKind, type ColumnTypeInput } from '../../utils/columnType'

const props = defineProps<ColumnTypeInput>()

const { t } = useI18n()

const kind = computed(() => columnTypeKind(props))
const title = computed(() => {
  const typeName = t(`type.${kind.value}`)
  const storage = props.storageType?.trim()
  return storage && storage !== typeName ? `${typeName} · ${storage}` : typeName
})
</script>

<template>
  <Tooltip :title="title">
    <span class="col-type-icon" :class="`is-${kind}`" :aria-label="title">
      <svg v-if="kind === 'datetime'" viewBox="0 0 16 16" aria-hidden="true">
        <rect x="2.5" y="3.5" width="11" height="10" rx="1.4" fill="none" stroke="currentColor" stroke-width="1.3" />
        <path d="M5 2.4v2.4M11 2.4v2.4M2.5 6.4h11" fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" />
        <rect x="5" y="8.2" width="2" height="2" rx="0.4" fill="currentColor" />
        <rect x="9" y="8.2" width="2" height="2" rx="0.4" fill="currentColor" />
      </svg>
      <span v-else class="col-type-glyph" aria-hidden="true">
        {{ kind === 'string' ? 'abc' : kind === 'integer' ? '123' : '1.2' }}
      </span>
    </span>
  </Tooltip>
</template>

<style scoped>
.col-type-icon {
  display: inline-flex;
  flex: 0 0 16px;
  align-items: center;
  justify-content: center;
  width: 16px;
  height: 16px;
  border-radius: 3px;
  background: color-mix(in srgb, currentColor 14%, transparent);
  line-height: 1;
  user-select: none;
}

.col-type-icon.is-string {
  color: #8b5cf6;
}

.col-type-icon.is-integer {
  color: #3b82f6;
}

.col-type-icon.is-number {
  color: #0d9488;
}

.col-type-icon.is-datetime {
  color: #16a34a;
}

:global([data-theme='dark']) .col-type-icon.is-string {
  color: #c4b5fd;
}

:global([data-theme='dark']) .col-type-icon.is-integer {
  color: #93c5fd;
}

:global([data-theme='dark']) .col-type-icon.is-number {
  color: #5eead4;
}

:global([data-theme='dark']) .col-type-icon.is-datetime {
  color: #86efac;
}

.col-type-glyph {
  font-family: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  font-size: 7px;
  font-weight: 700;
  letter-spacing: -0.04em;
}

svg {
  display: block;
  width: 12px;
  height: 12px;
}
</style>
