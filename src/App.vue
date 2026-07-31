<script setup lang="ts">
import { getCurrentWindow } from '@tauri-apps/api/window'

import PetWindow from './pet/PetWindow.vue'
import SettingsRoot from './settings/SettingsRoot.vue'
import { isTauriRuntime } from './shared/runtime'

function detectView(): 'pet' | 'settings' {
  if (isTauriRuntime()) return getCurrentWindow().label === 'pet' ? 'pet' : 'settings'
  return new URLSearchParams(window.location.search).get('view') === 'pet' ? 'pet' : 'settings'
}

const view = detectView()
document.documentElement.dataset.view = view
</script>

<template>
  <PetWindow v-if="view === 'pet'" />
  <SettingsRoot v-else />
</template>
