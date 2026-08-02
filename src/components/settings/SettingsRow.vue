<script setup lang="ts">
import { computed } from 'vue'

import type { SettingsIconName } from '../../settings/icons'
import AppIcon from './AppIcon.vue'

const props = defineProps<{
  title: string
  description?: string
  disabled?: boolean
  controlId?: string
  icon?: SettingsIconName
}>()

const descriptionId = computed(() => props.controlId && props.description ? `${props.controlId}-description` : undefined)
</script>

<template>
  <section class="settings-row" :class="{ 'is-disabled': disabled }">
    <span v-if="icon" class="settings-row__icon"><AppIcon :name="icon" /></span>
    <div class="settings-row__copy">
      <label v-if="controlId" class="settings-row__title" :for="controlId">{{ title }}</label>
      <h3 v-else>{{ title }}</h3>
      <p v-if="description" :id="descriptionId">{{ description }}</p>
    </div>
    <div class="settings-row__control"><slot :description-id="descriptionId" /></div>
  </section>
</template>
