<script setup lang="ts">
import type { SettingsAboutInfo } from '../../settings/types'
import AppIcon from './AppIcon.vue'

defineProps<{
  info: SettingsAboutInfo
}>()

const emit = defineEmits<{
  openHomepage: [url: string]
}>()
</script>

<template>
  <div class="about-panel">
    <div class="about-panel__card">
      <div class="about-panel__logo"><AppIcon name="sparkles" :size="30" /></div>
      <p class="settings-eyebrow">ABOUT QPETS</p>
      <h1>{{ info.appName }}</h1>
      <p class="about-panel__version">版本 {{ info.version }}</p>
      <p class="about-panel__description">{{ info.description ?? '把一只轻巧、安静的小伙伴放到你的桌面上。' }}</p>
      <div class="about-panel__traits" aria-label="项目特点">
        <span>本地角色库</span>
        <span>轻量陪伴</span>
        <span>开源项目</span>
      </div>
      <button v-if="info.homepage" type="button" class="settings-button settings-button--primary" @click="emit('openHomepage', info.homepage)">
        访问项目主页
        <AppIcon name="external" :size="16" />
      </button>
    </div>
  </div>
</template>
