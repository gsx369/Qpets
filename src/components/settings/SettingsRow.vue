<script setup lang="ts">
defineProps<{
  title: string
  description?: string
  disabled?: boolean
}>()
</script>

<template>
  <section class="settings-row" :class="{ 'is-disabled': disabled }">
    <div class="settings-row__copy">
      <h3>{{ title }}</h3>
      <p v-if="description">{{ description }}</p>
    </div>
    <div class="settings-row__control"><slot /></div>
  </section>
</template>
