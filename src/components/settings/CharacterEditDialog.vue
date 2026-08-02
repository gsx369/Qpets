<script setup lang="ts">
import { computed, nextTick, ref, watch } from 'vue'

import {
  CHARACTER_DESCRIPTION_LIMIT,
  CHARACTER_NAME_LIMIT,
  characterLength,
  normalizeCharacterMetadata,
  validateCharacterMetadata,
} from '../../settings/character-metadata'
import type { CharacterMetadataDraft, PetCharacter } from '../../settings/types'
import AppIcon from './AppIcon.vue'

const props = defineProps<{
  character?: PetCharacter
  saving?: boolean
  inactive?: boolean
}>()

const emit = defineEmits<{
  cancel: []
  save: [metadata: CharacterMetadataDraft]
  reset: []
}>()

const nameInput = ref<HTMLInputElement>()
const dialog = ref<HTMLFormElement>()
const displayName = ref('')
const description = ref('')
let returnFocus: HTMLElement | undefined

watch(() => props.character?.id, async (characterId, previousId) => {
  if (!characterId) {
    const target = returnFocus
    returnFocus = undefined
    await nextTick()
    if (target?.isConnected) target.focus()
    return
  }
  const character = props.character
  if (!character) return
  if (!previousId && document.activeElement instanceof HTMLElement) {
    returnFocus = document.activeElement
  }
  displayName.value = character.name
  description.value = character.description
  await nextTick()
  nameInput.value?.focus()
  nameInput.value?.select()
}, { immediate: true })

const normalized = computed(() => normalizeCharacterMetadata({
  displayName: displayName.value,
  description: description.value,
}))
const validationError = computed(() => validateCharacterMetadata(normalized.value))
const changed = computed(() => Boolean(props.character)
  && (normalized.value.displayName !== props.character?.name
    || normalized.value.description !== props.character?.description))

function save() {
  if (!validationError.value && changed.value) emit('save', normalized.value)
}

function trapFocus(event: KeyboardEvent) {
  if (props.inactive) return
  const focusable = Array.from(dialog.value?.querySelectorAll<HTMLElement>(
    'button:not([disabled]), input:not([disabled]), textarea:not([disabled]), [tabindex]:not([tabindex="-1"])',
  ) ?? [])
  if (focusable.length === 0) {
    event.preventDefault()
    return
  }
  const first = focusable[0]
  const last = focusable[focusable.length - 1]
  const active = document.activeElement
  if (event.shiftKey && (active === first || !dialog.value?.contains(active))) {
    event.preventDefault()
    last.focus()
  } else if (!event.shiftKey && (active === last || !dialog.value?.contains(active))) {
    event.preventDefault()
    first.focus()
  }
}
</script>

<template>
  <div
    v-if="character"
    class="settings-dialog-backdrop"
    role="presentation"
    :inert="inactive || undefined"
    :aria-hidden="inactive ? 'true' : undefined"
    @click.self="!inactive && !saving && emit('cancel')"
    @keydown.esc.stop="!inactive && !saving && emit('cancel')"
  >
    <form
      ref="dialog"
      class="settings-dialog settings-dialog--edit"
      role="dialog"
      :aria-modal="inactive ? undefined : 'true'"
      aria-labelledby="character-edit-title"
      aria-describedby="character-edit-description"
      @keydown.tab="trapFocus"
      @submit.prevent="save"
    >
      <div class="settings-dialog__icon settings-dialog__icon--accent"><AppIcon name="edit" /></div>
      <div class="settings-dialog__heading">
        <h2 id="character-edit-title">编辑角色资料</h2>
        <p id="character-edit-description">名称和介绍只保存在本机，不会修改原始角色模型。</p>
      </div>

      <div class="character-form">
        <label for="character-display-name">角色名称</label>
        <div class="character-form__field">
          <input
            id="character-display-name"
            ref="nameInput"
            v-model="displayName"
            type="text"
            autocomplete="off"
            :disabled="saving"
            :aria-invalid="Boolean(validationError)"
          />
          <span :class="{ 'is-over-limit': characterLength(displayName) > CHARACTER_NAME_LIMIT }">
            {{ characterLength(displayName) }}/{{ CHARACTER_NAME_LIMIT }}
          </span>
        </div>

        <label for="character-description">角色介绍</label>
        <div class="character-form__field character-form__field--textarea">
          <textarea id="character-description" v-model="description" rows="4" :disabled="saving" />
          <span :class="{ 'is-over-limit': characterLength(description) > CHARACTER_DESCRIPTION_LIMIT }">
            {{ characterLength(description) }}/{{ CHARACTER_DESCRIPTION_LIMIT }}
          </span>
        </div>
        <p v-if="validationError" class="character-form__error" role="alert">{{ validationError }}</p>
      </div>

      <footer class="settings-dialog__footer settings-dialog__footer--split">
        <button
          v-if="character.metadataCustomized"
          type="button"
          class="settings-button settings-button--quiet"
          :disabled="saving"
          @click="emit('reset')"
        >
          <AppIcon name="reset" :size="16" />
          恢复默认
        </button>
        <span v-else />
        <span class="settings-dialog__footer-actions">
          <button type="button" class="settings-button settings-button--ghost" :disabled="saving" @click="emit('cancel')">取消</button>
          <button type="submit" class="settings-button settings-button--primary" :disabled="saving || Boolean(validationError) || !changed">
            {{ saving ? '正在保存…' : '保存修改' }}
          </button>
        </span>
      </footer>
    </form>
  </div>
</template>
