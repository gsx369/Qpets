<script setup lang="ts">
import { computed, onMounted } from 'vue'

import { resolveAssetUrl, useAppRuntime } from '../shared/runtime'
import type { AppSettings } from '../shared/types'
import SettingsPage from './SettingsPage.vue'
import type { PetCharacter } from './types'

const runtime = useAppRuntime()

const characters = computed<PetCharacter[]>(() => runtime.state.pets.map((pet) => ({
  id: pet.id,
  name: pet.displayName,
  cover: resolveAssetUrl(pet.thumbnailPath),
  builtIn: pet.source === 'builtin',
  active: pet.id === runtime.state.selectedPetId,
  description: pet.description,
  modelType: pet.renderType === 'sprite-atlas-v2' ? '动态' : '静态',
})))

function selectCharacter(character: PetCharacter) {
  void runtime.selectPet(character.id)
}

function updateInteractionSettings(settings: AppSettings) {
  void runtime.updateSettings(settings)
}

onMounted(() => void runtime.initialize())
</script>

<template>
  <div class="settings-root">
    <SettingsPage
      :characters="characters"
      :interaction-settings="runtime.state.settings"
      :about="{
        appName: 'Qpets',
        version: '0.1.0',
        description: '晴檐、堇语与糖葫芦陪伴你的轻量桌面宠物。',
        homepage: 'https://github.com/gsx369/Qpets',
      }"
      @select-character="selectCharacter"
      @add-image="runtime.addStaticPet"
      @import-package="runtime.importPetPackage"
      @delete-character="runtime.deletePet($event.id)"
      @update-interaction-settings="updateInteractionSettings"
      @open-homepage="runtime.visitHomepage"
    />

    <div v-if="runtime.busy.value" class="settings-status" role="status">正在更新角色库…</div>
    <button v-if="runtime.error.value" type="button" class="settings-error" @click="runtime.dismissError">
      {{ runtime.error.value }}
    </button>
  </div>
</template>
