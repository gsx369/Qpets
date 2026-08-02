<script setup lang="ts">
import { nextTick, ref, watch } from 'vue'

import AppIcon from './AppIcon.vue'

const props = withDefaults(defineProps<{
  open: boolean
  title: string
  description: string
  confirmLabel?: string
  tone?: 'danger' | 'neutral'
  busy?: boolean
  busyLabel?: string
}>(), {
  confirmLabel: '删除角色',
  tone: 'danger',
  busy: false,
  busyLabel: '正在处理…',
})

const emit = defineEmits<{
  cancel: []
  confirm: []
}>()

const cancelButton = ref<HTMLButtonElement>()
const dialog = ref<HTMLElement>()
let returnFocus: HTMLElement | undefined
watch(() => props.open, async (open) => {
  if (open) {
    if (document.activeElement instanceof HTMLElement) returnFocus = document.activeElement
    await nextTick()
    cancelButton.value?.focus()
  } else {
    const target = returnFocus
    returnFocus = undefined
    await nextTick()
    if (target?.isConnected) target.focus()
  }
})

watch(() => props.busy, async (busy) => {
  if (!props.open) return
  await nextTick()
  if (busy) dialog.value?.focus()
  else cancelButton.value?.focus()
})

function trapFocus(event: KeyboardEvent) {
  const focusable = Array.from(dialog.value?.querySelectorAll<HTMLElement>(
    'button:not([disabled]), [tabindex]:not([tabindex="-1"])',
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
  <div v-if="open" class="settings-dialog-backdrop" role="presentation" @click.self="!busy && emit('cancel')" @keydown.esc.stop="!busy && emit('cancel')">
    <section ref="dialog" class="settings-dialog" role="alertdialog" aria-modal="true" :aria-busy="busy" aria-labelledby="confirm-dialog-title" aria-describedby="confirm-dialog-description" tabindex="-1" @keydown.tab="trapFocus">
      <div class="settings-dialog__icon" :class="{ 'settings-dialog__icon--accent': tone === 'neutral' }">
        <AppIcon :name="tone === 'danger' ? 'alert' : 'reset'" :size="17" />
      </div>
      <div>
        <h2 id="confirm-dialog-title">{{ title }}</h2>
        <p id="confirm-dialog-description">{{ description }}</p>
      </div>
      <footer>
        <button ref="cancelButton" type="button" class="settings-button settings-button--ghost" :disabled="busy" @click="emit('cancel')">取消</button>
        <button
          type="button"
          class="settings-button"
          :class="tone === 'danger' ? 'settings-button--danger' : 'settings-button--primary'"
          :disabled="busy"
          @click="emit('confirm')"
        >{{ busy ? busyLabel : confirmLabel }}</button>
      </footer>
    </section>
  </div>
</template>
