<script setup lang="ts">
import { computed, ref } from 'vue'

import AboutPanel from '../components/settings/AboutPanel.vue'
import CharacterLibrary from '../components/settings/CharacterLibrary.vue'
import PetInteractionPanel from '../components/settings/PetInteractionPanel.vue'
import SettingsSidebar from '../components/settings/SettingsSidebar.vue'
import type { CharacterMetadataDraft, PetCharacter, PetInteractionSettings, SettingsAboutInfo, SettingsSection } from './types'

import '../styles/settings.css'

const props = withDefaults(defineProps<{
  characters?: PetCharacter[]
  interactionSettings?: PetInteractionSettings
  about?: SettingsAboutInfo
  initialSection?: SettingsSection
}>(), {
  characters: () => [
    { id: 'qpet-z1-sunny-brim', name: '赵大美女', builtIn: true, active: true, description: '大美女，大设计师，P图大师，复合型人才', metadataCustomized: false, modelType: '动态' },
    { id: 'qpet-z2-violet-whisper', name: '赵二美女', builtIn: true, description: '二美女，二设计师，P图二师，复合型人才', metadataCustomized: false, modelType: '动态' },
    { id: 'qpet-gu-candied-haw', name: '俺滴老妹', builtIn: true, description: '世界上最可爱的人', metadataCustomized: false, modelType: '动态' },
  ],
  interactionSettings: () => ({ alwaysOnTop: true, clickThrough: false, followCursor: true, showBubble: true, startWithWindows: false, scale: 1, idleIntervalSeconds: 30 }),
  about: () => ({ appName: 'Qpets', version: '0.1.3', description: '让每一次打开电脑，都有一位小伙伴在身边。' }),
  initialSection: 'library',
})

const activeSection = ref<SettingsSection>(props.initialSection)
const settingsContent = ref<HTMLElement>()
const normalizedCharacters = computed(() => props.characters)

const emit = defineEmits<{
  selectCharacter: [character: PetCharacter]
  addImage: []
  importPackage: []
  deleteCharacter: [character: PetCharacter]
  updateCharacterMetadata: [character: PetCharacter, metadata: CharacterMetadataDraft, complete: (saved: boolean) => void]
  resetCharacterMetadata: [character: PetCharacter, complete: (saved: boolean) => void]
  updateInteractionSettings: [settings: PetInteractionSettings]
  openHomepage: [url: string]
  sectionChange: [section: SettingsSection]
}>()

function selectSection(section: SettingsSection) {
  activeSection.value = section
  if (settingsContent.value) settingsContent.value.scrollTop = 0
  emit('sectionChange', section)
}

function updateCharacterMetadata(
  character: PetCharacter,
  metadata: CharacterMetadataDraft,
  complete: (saved: boolean) => void,
) {
  emit('updateCharacterMetadata', character, metadata, complete)
}

function resetCharacterMetadata(character: PetCharacter, complete: (saved: boolean) => void) {
  emit('resetCharacterMetadata', character, complete)
}
</script>

<template>
  <main class="settings-shell">
    <SettingsSidebar :active="activeSection" @select="selectSection" />
    <div ref="settingsContent" class="settings-content">
      <CharacterLibrary
        v-if="activeSection === 'library'"
        :characters="normalizedCharacters"
        @select="emit('selectCharacter', $event)"
        @add-image="emit('addImage')"
        @import-package="emit('importPackage')"
        @delete="emit('deleteCharacter', $event)"
        @update-metadata="updateCharacterMetadata"
        @reset-metadata="resetCharacterMetadata"
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
