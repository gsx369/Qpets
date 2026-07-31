<script setup lang="ts">
import type { SettingsAboutInfo } from '../../settings/types'

defineProps<{
  info: SettingsAboutInfo
}>()

const emit = defineEmits<{
  openHomepage: [url: string]
}>()
</script>

<template>
  <div class="about-panel">
    <div class="about-panel__logo">Q</div>
    <p class="settings-eyebrow">ABOUT QPETS</p>
    <h1>{{ info.appName }}</h1>
    <p class="about-panel__version">版本 {{ info.version }}</p>
    <p class="about-panel__description">{{ info.description ?? '把一只轻巧、安静的小伙伴放到你的桌面上。' }}</p>
    <button v-if="info.homepage" type="button" class="settings-button settings-button--secondary" @click="emit('openHomepage', info.homepage)">
      访问项目主页 ↗
    </button>
  </div>
</template>
