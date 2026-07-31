<script setup lang="ts">
import type { SettingsSection } from '../../settings/types'

defineProps<{
  active: SettingsSection
}>()

const emit = defineEmits<{
  select: [section: SettingsSection]
}>()

const items: Array<{ id: SettingsSection; icon: string; label: string }> = [
  { id: 'library', icon: '✦', label: '角色库' },
  { id: 'pet', icon: '⌁', label: '桌宠与互动' },
  { id: 'about', icon: 'i', label: '关于' },
]
</script>

<template>
  <aside class="settings-sidebar">
    <div class="settings-brand">
      <div class="settings-brand__mark">Q</div>
      <div>
        <strong>Qpets</strong>
        <span>桌面小伙伴</span>
      </div>
    </div>

    <nav class="settings-nav" aria-label="设置导航">
      <button
        v-for="item in items"
        :key="item.id"
        type="button"
        class="settings-nav__item"
        :class="{ 'is-active': active === item.id }"
        @click="emit('select', item.id)"
      >
        <span class="settings-nav__icon">{{ item.icon }}</span>
        {{ item.label }}
      </button>
    </nav>

    <p class="settings-sidebar__hint">所有更改会立即生效</p>
  </aside>
</template>
