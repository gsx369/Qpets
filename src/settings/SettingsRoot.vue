<script setup lang="ts">
import { computed, onBeforeUnmount, onMounted } from 'vue'

import { resolveAssetUrl, useAppRuntime } from '../shared/runtime'
import type { AppSettings } from '../shared/types'
import AppIcon from '../components/settings/AppIcon.vue'
import SettingsPage from './SettingsPage.vue'
import type { CharacterMetadataDraft, PetCharacter } from './types'

const runtime = useAppRuntime()

const characters = computed<PetCharacter[]>(() => runtime.state.pets.map((pet) => ({
  id: pet.id,
  name: pet.displayName,
  cover: resolveAssetUrl(pet.thumbnailPath),
  builtIn: pet.source === 'builtin',
  active: pet.id === runtime.state.selectedPetId,
  description: pet.description,
  metadataCustomized: pet.metadataCustomized,
  modelType: pet.renderType === 'sprite-atlas-v2' ? '动态' : '静态',
})))

function selectCharacter(character: PetCharacter) {
  void runtime.selectPet(character.id)
}

function updateInteractionSettings(settings: Omit<AppSettings, 'volume'>) {
  // `volume` remains in the backend transport schema for persisted-state
  // compatibility, but Qpets currently has no audio output to control.
  void runtime.updateSettings({ ...runtime.state.settings, ...settings })
}

async function updateCharacterMetadata(
  character: PetCharacter,
  metadata: CharacterMetadataDraft,
  complete: (saved: boolean) => void,
) {
  complete(await runtime.updatePetMetadata(character.id, metadata))
}

async function resetCharacterMetadata(character: PetCharacter, complete: (saved: boolean) => void) {
  complete(await runtime.resetPetMetadata(character.id))
}

onMounted(() => void runtime.initialize())
onBeforeUnmount(() => runtime.dispose())
</script>

<template>
  <div class="settings-root">
    <div v-if="!runtime.ready.value" class="settings-loading" role="status" aria-live="polite">
      正在加载设置…
    </div>
    <SettingsPage
      v-else
      :characters="characters"
      :interaction-settings="runtime.state.settings"
      :about="{
        appName: 'Qpets',
        version: '0.1.2',
        description: '晴檐、堇语与糖葫芦陪伴你的轻量桌面宠物。',
        homepage: 'https://github.com/gsx369/Qpets',
      }"
      @select-character="selectCharacter"
      @add-image="runtime.addStaticPet"
      @import-package="runtime.importPetPackage"
      @delete-character="runtime.deletePet($event.id)"
      @update-character-metadata="updateCharacterMetadata"
      @reset-character-metadata="resetCharacterMetadata"
      @update-interaction-settings="updateInteractionSettings"
      @open-homepage="runtime.visitHomepage"
    />

    <div v-if="runtime.busy.value" class="settings-status" role="status">正在保存更改…</div>
    <div v-if="runtime.error.value" class="settings-error" role="alert">
      <span>{{ runtime.error.value }}</span>
      <button type="button" aria-label="关闭错误提示" @click="runtime.dismissError">
        <AppIcon name="close" :size="15" />
      </button>
    </div>
  </div>
</template>
