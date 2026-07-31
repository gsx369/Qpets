<script setup lang="ts">
import { computed, ref, watch } from 'vue'

import type { CharacterMetadataDraft, PetCharacter } from '../../settings/types'
import AppIcon from './AppIcon.vue'
import CharacterEditDialog from './CharacterEditDialog.vue'
import ConfirmDialog from './ConfirmDialog.vue'

const props = defineProps<{
  characters: PetCharacter[]
}>()

const emit = defineEmits<{
  select: [character: PetCharacter]
  addImage: []
  importPackage: []
  delete: [character: PetCharacter]
  updateMetadata: [character: PetCharacter, metadata: CharacterMetadataDraft, complete: (saved: boolean) => void]
  resetMetadata: [character: PetCharacter, complete: (saved: boolean) => void]
}>()

const pendingDeletionId = ref<string>()
const editingId = ref<string>()
const resetRequested = ref(false)
const savingMetadata = ref(false)
const pendingDeletion = computed(() => props.characters.find(character => character.id === pendingDeletionId.value))
const editingCharacter = computed(() => props.characters.find(character => character.id === editingId.value))

watch(() => props.characters, (characters) => {
  if (pendingDeletionId.value && !characters.some(character => character.id === pendingDeletionId.value)) {
    pendingDeletionId.value = undefined
  }
  if (editingId.value && !characters.some(character => character.id === editingId.value)) {
    editingId.value = undefined
    resetRequested.value = false
    savingMetadata.value = false
  }
})

function confirmDelete() {
  if (pendingDeletion.value) emit('delete', pendingDeletion.value)
  pendingDeletionId.value = undefined
}

function saveMetadata(metadata: CharacterMetadataDraft) {
  const character = editingCharacter.value
  if (!character || savingMetadata.value) return
  savingMetadata.value = true
  emit('updateMetadata', character, metadata, (saved) => {
    savingMetadata.value = false
    if (saved && editingId.value === character.id) editingId.value = undefined
  })
}

function closeEditor() {
  if (savingMetadata.value) return
  editingId.value = undefined
  resetRequested.value = false
}

function requestReset() {
  if (editingCharacter.value && !savingMetadata.value) resetRequested.value = true
}

function confirmReset() {
  const character = editingCharacter.value
  if (!character || savingMetadata.value) return
  savingMetadata.value = true
  emit('resetMetadata', character, (saved) => {
    savingMetadata.value = false
    if (saved) {
      resetRequested.value = false
      if (editingId.value === character.id) editingId.value = undefined
    }
  })
}
</script>

<template>
  <div class="settings-page-heading">
    <div>
      <p class="settings-eyebrow">CHARACTER LIBRARY</p>
      <h1>角色库</h1>
      <p>选择、整理并编辑你的小伙伴，所有资料只保存在本机。</p>
    </div>
    <div class="settings-actions">
      <button type="button" class="settings-button settings-button--secondary" @click="emit('addImage')">
        <AppIcon name="image-plus" :size="17" />
        添加图片
      </button>
      <button type="button" class="settings-button settings-button--primary" @click="emit('importPackage')">
        <AppIcon name="package" :size="17" />
        导入角色包
      </button>
    </div>
  </div>

  <div class="character-grid">
    <article
      v-for="character in characters"
      :key="character.id"
      class="character-card"
      :class="{ 'is-active': character.active }"
    >
      <button
        type="button"
        class="character-card__select"
        :aria-pressed="Boolean(character.active)"
        :aria-label="`${character.active ? '当前角色' : '选择角色'}：${character.name}`"
        @click="emit('select', character)"
      >
        <span class="character-card__cover" :class="{ 'has-image': character.cover }">
          <img v-if="character.cover" :src="character.cover" alt="" draggable="false" />
          <span v-else class="character-card__initial">{{ character.name.slice(0, 1) }}</span>
          <span v-if="character.active" class="character-card__active">
            <AppIcon name="check" :size="12" />
            正在使用
          </span>
          <span v-if="character.builtIn" class="character-card__tag">内置</span>
          <span v-if="character.modelType" class="character-card__type">{{ character.modelType }}</span>
        </span>
        <span class="character-card__body">
          <strong>{{ character.name }}</strong>
          <span>{{ character.description || '还没有角色介绍' }}</span>
        </span>
      </button>

      <div class="character-card__tools">
        <button
          type="button"
          class="character-card__tool"
          :aria-label="`编辑 ${character.name} 的名称和介绍`"
          title="编辑名称和介绍"
          @click="editingId = character.id"
        >
          <AppIcon name="edit" :size="16" />
        </button>
        <button
          v-if="!character.builtIn"
          type="button"
          class="character-card__tool character-card__tool--danger"
          :aria-label="`删除 ${character.name}`"
          title="删除角色"
          @click="pendingDeletionId = character.id"
        >
          <AppIcon name="trash" :size="16" />
        </button>
      </div>
    </article>

    <button type="button" class="character-add-card" @click="emit('addImage')">
      <span class="character-add-card__icon"><AppIcon name="plus" :size="22" /></span>
      <strong>添加新角色</strong>
      <small>支持透明 PNG、WebP</small>
    </button>
  </div>

  <p class="settings-page-note">
    名称和介绍均可编辑；内置角色受保护不能删除，自定义角色可以随时移除。
  </p>

  <CharacterEditDialog
    :character="editingCharacter"
    :saving="savingMetadata"
    :inactive="resetRequested"
    @cancel="closeEditor"
    @save="saveMetadata"
    @reset="requestReset"
  />

  <ConfirmDialog
    :open="resetRequested && Boolean(editingCharacter)"
    title="恢复默认资料？"
    :description="`“${editingCharacter?.name ?? ''}”的自定义名称和介绍将恢复为角色包中的默认内容。`"
    confirm-label="恢复默认"
    tone="neutral"
    :busy="savingMetadata"
    busy-label="正在恢复…"
    @cancel="resetRequested = false"
    @confirm="confirmReset"
  />

  <ConfirmDialog
    :open="Boolean(pendingDeletion)"
    title="删除自定义角色？"
    :description="`“${pendingDeletion?.name ?? ''}”将从角色库中移除，此操作无法撤销。`"
    @cancel="pendingDeletionId = undefined"
    @confirm="confirmDelete"
  />
</template>
