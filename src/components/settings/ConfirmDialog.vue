<script setup lang="ts">
defineProps<{
  open: boolean
  title: string
  description: string
}>()

const emit = defineEmits<{
  cancel: []
  confirm: []
}>()
</script>

<template>
  <div v-if="open" class="settings-dialog-backdrop" role="presentation" @click.self="emit('cancel')">
    <section class="settings-dialog" role="alertdialog" aria-modal="true" :aria-label="title">
      <div class="settings-dialog__icon">!</div>
      <div>
        <h2>{{ title }}</h2>
        <p>{{ description }}</p>
      </div>
      <footer>
        <button type="button" class="settings-button settings-button--ghost" @click="emit('cancel')">取消</button>
        <button type="button" class="settings-button settings-button--danger" @click="emit('confirm')">删除角色</button>
      </footer>
    </section>
  </div>
</template>
