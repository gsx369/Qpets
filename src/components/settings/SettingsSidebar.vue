<script setup lang="ts">
import type { SettingsSection } from '../../settings/types'
import type { SettingsIconName } from '../../settings/icons'
import AppIcon from './AppIcon.vue'

defineProps<{
  active: SettingsSection
}>()

const emit = defineEmits<{
  select: [section: SettingsSection]
}>()

const items: Array<{ id: SettingsSection; icon: SettingsIconName; label: string }> = [
  { id: 'library', icon: 'library', label: '角色库' },
  { id: 'pet', icon: 'sliders', label: '桌宠与互动' },
  { id: 'about', icon: 'info', label: '关于' },
]
</script>

<template>
  <aside class="settings-sidebar">
    <div class="settings-brand">
      <div class="settings-brand__mark" aria-hidden="true">Q</div>
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
        :aria-current="active === item.id ? 'page' : undefined"
        :title="item.label"
        @click="emit('select', item.id)"
      >
        <span class="settings-nav__icon"><AppIcon :name="item.icon" /></span>
        <span class="settings-nav__label">{{ item.label }}</span>
      </button>
    </nav>

    <p class="settings-sidebar__hint">所有更改会立即生效</p>
  </aside>
</template>
