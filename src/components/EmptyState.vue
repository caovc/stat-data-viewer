<script setup lang="ts">
import { useI18n } from 'vue-i18n'
import { FolderOpenOutlined, InboxOutlined } from '@antdv-next/icons'
import { Button, Empty, Flex, Space, TypographyParagraph, TypographyText, theme } from 'antdv-next'

const emit = defineEmits<{
  open: []
}>()

const { t } = useI18n()
const { token } = theme.useToken()
</script>

<template>
  <Flex class="empty-state" align="center" justify="center" vertical>
    <Empty>
      <template #image>
        <InboxOutlined :style="{ fontSize: '48px', color: token.colorPrimary }" />
      </template>
      <template #description>
        <Flex vertical align="center" gap="small">
          <TypographyText strong class="empty-title">{{ t('empty.title') }}</TypographyText>
          <TypographyParagraph type="secondary" class="empty-copy">
            {{ t('empty.copy') }}
          </TypographyParagraph>
        </Flex>
      </template>
      <Space>
        <Button type="primary" @click="emit('open')">
          <template #icon><FolderOpenOutlined /></template>
          {{ t('empty.open') }}
        </Button>
      </Space>
    </Empty>
    <TypographyText type="secondary" class="empty-keys">
      {{ t('empty.keys') }}
    </TypographyText>
  </Flex>
</template>

<style scoped>
.empty-state {
  height: 100%;
  padding: 32px;
}

.empty-title {
  font-size: 18px;
  text-wrap: balance;
}

.empty-copy {
  max-width: 42ch;
  margin-bottom: 0;
  text-align: center;
  text-wrap: pretty;
}

.empty-keys {
  margin-top: 16px;
  font-size: 12px;
}
</style>
