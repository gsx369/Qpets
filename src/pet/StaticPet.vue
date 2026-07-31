<script setup lang="ts">
import { computed } from 'vue'

import type { PetAction } from '../shared/types'

const props = defineProps<{
  source: string
  name: string
  action: PetAction
  lookIndex?: number
}>()

const gazeStyle = computed(() => {
  if (props.lookIndex === undefined) return undefined
  const radians = (props.lookIndex * 22.5 * Math.PI) / 180
  return {
    '--gaze-x': `${Math.sin(radians) * 5}px`,
    '--gaze-y': `${-Math.cos(radians) * 4}px`,
    '--gaze-rotate': `${Math.sin(radians) * 2.2}deg`,
  }
})
</script>

<template>
  <div class="static-pet" :class="`static-pet--${action}`" :style="gazeStyle">
    <img v-if="source" :src="source" :alt="name" draggable="false" />
    <div v-else class="pet-placeholder" aria-hidden="true">{{ name.slice(0, 1) }}</div>
  </div>
</template>
