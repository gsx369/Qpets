<script setup lang="ts">
import { computed, ref } from 'vue'

import AboutPanel from '../components/settings/AboutPanel.vue'
import CharacterLibrary from '../components/settings/CharacterLibrary.vue'
import PetInteractionPanel from '../components/settings/PetInteractionPanel.vue'
import SettingsSidebar from '../components/settings/SettingsSidebar.vue'
import type { PetCharacter, PetInteractionSettings, SettingsAboutInfo, SettingsSection } from './types'

import '../styles/settings.css'

const props = withDefaults(defineProps<{
  characters?: PetCharacter[]
  interactionSettings?: PetInteractionSettings
  about?: SettingsAboutInfo
  initialSection?: SettingsSection
}>(), {
  characters: () => [
    { id: 'qpet-z1-sunny-brim', name: '晴檐', builtIn: true, active: true, description: '戴草帽的温柔桌面伙伴', modelType: '动态' },
    { id: 'qpet-z2-violet-whisper', name: '堇语', builtIn: true, description: '藏着一枝小花的安静伙伴', modelType: '动态' },
    { id: 'qpet-gu-candied-haw', name: '糖葫芦', builtIn: true, description: '橙金汉服风格的桌面伙伴', modelType: '静态' },
  ],
  interactionSettings: () => ({ alwaysOnTop: true, clickThrough: false, followCursor: true, showBubble: true, startWithWindows: false, volume: 0.55, scale: 1, idleIntervalSeconds: 30 }),
  about: () => ({ appName: 'Qpets', version: '0.1.0', description: '让每一次打开电脑，都有一位小伙伴在身边。' }),
  initialSection: 'library',
})

const activeSection = ref<SettingsSection>(props.initialSection)
const normalizedCharacters = computed(() => props.characters)

const emit = defineEmits<{
  selectCharacter: [character: PetCharacter]
  addImage: []
  importPackage: []
  deleteCharacter: [character: PetCharacter]
  updateInteractionSettings: [settings: PetInteractionSettings]
  openHomepage: [url: string]
  sectionChange: [section: SettingsSection]
}>()

function selectSection(section: SettingsSection) {
  activeSection.value = section
  emit('sectionChange', section)
}
</script>

<template>
  <main class="settings-shell">
    <SettingsSidebar :active="activeSection" @select="selectSection" />
    <div class="settings-content">
      <CharacterLibrary
        v-if="activeSection === 'library'"
        :characters="normalizedCharacters"
        @select="emit('selectCharacter', $event)"
        @add-image="emit('addImage')"
        @import-package="emit('importPackage')"
        @delete="emit('deleteCharacter', $event)"
      />
      <PetInteractionPanel
        v-else-if="activeSection === 'pet'"
        :settings="interactionSettings"
        @update="emit('updateInteractionSettings', $event)"
      />
      <AboutPanel
        v-else
        :info="about"
        @open-homepage="emit('openHomepage', $event)"
      />
    </div>
  </main>
</template>
