<script setup lang="ts">
import { computed, ref } from 'vue'

import type { PetCharacter } from '../../settings/types'
import ConfirmDialog from './ConfirmDialog.vue'

const props = defineProps<{
  characters: PetCharacter[]
}>()

const emit = defineEmits<{
  select: [character: PetCharacter]
  addImage: []
  importPackage: []
  delete: [character: PetCharacter]
}>()

const pendingDeletion = ref<PetCharacter>()
const orderedCharacters = computed(() => [...props.characters].sort((a, b) => Number(Boolean(b.active)) - Number(Boolean(a.active))))

function requestDelete(character: PetCharacter) {
  pendingDeletion.value = character
}

function confirmDelete() {
  if (pendingDeletion.value) emit('delete', pendingDeletion.value)
  pendingDeletion.value = undefined
}
</script>

<template>
  <div class="settings-page-heading">
    <div>
      <p class="settings-eyebrow">CHARACTER LIBRARY</p>
      <h1>角色库</h1>
      <p>选择一位小伙伴，让它陪你待在桌面上。</p>
    </div>
    <div class="settings-actions">
      <button type="button" class="settings-button settings-button--secondary" @click="emit('addImage')">＋ 添加图片</button>
      <button type="button" class="settings-button settings-button--primary" @click="emit('importPackage')">导入角色包</button>
    </div>
  </div>

  <div class="character-grid">
    <article
      v-for="character in orderedCharacters"
      :key="character.id"
      class="character-card"
      :class="{ 'is-active': character.active }"
      tabindex="0"
      @click="emit('select', character)"
      @keydown.enter="emit('select', character)"
    >
      <div class="character-card__cover" :class="{ 'has-image': character.cover }">
        <img v-if="character.cover" :src="character.cover" :alt="character.name" />
        <span v-else>{{ character.name.slice(0, 1) }}</span>
        <span v-if="character.active" class="character-card__active">正在使用</span>
        <span v-if="character.builtIn" class="character-card__tag">内置</span>
        <span v-if="character.modelType" class="character-card__type">{{ character.modelType }}</span>
      </div>
      <div class="character-card__body">
        <div>
          <h2>{{ character.name }}</h2>
          <p>{{ character.description ?? (character.builtIn ? 'Qpets 内置角色' : '自定义角色') }}</p>
        </div>
        <button
          v-if="!character.builtIn"
          type="button"
          class="character-card__delete"
          :aria-label="`删除 ${character.name}`"
          title="删除角色"
          @click.stop="requestDelete(character)"
        >
          ×
        </button>
      </div>
    </article>

    <button type="button" class="character-add-card" @click="emit('addImage')">
      <span>＋</span>
      <strong>添加新角色</strong>
      <small>支持透明 PNG、WebP</small>
    </button>
  </div>

  <p class="settings-page-note">内置角色受保护，不能删除。导入的角色包将由应用验证后加入角色库。</p>

  <ConfirmDialog
    :open="Boolean(pendingDeletion)"
    title="删除自定义角色？"
    :description="`“${pendingDeletion?.name ?? ''}”将从角色库中移除，此操作无法撤销。`"
    @cancel="pendingDeletion = undefined"
    @confirm="confirmDelete"
  />
</template>
