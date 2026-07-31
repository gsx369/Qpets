<script setup lang="ts">
import type { PetInteractionSettings } from '../../settings/types'
import SettingsRow from './SettingsRow.vue'

defineProps<{
  settings: PetInteractionSettings
}>()

const emit = defineEmits<{
  update: [settings: PetInteractionSettings]
}>()

function change<K extends keyof PetInteractionSettings>(settings: PetInteractionSettings, key: K, value: PetInteractionSettings[K]) {
  emit('update', { ...settings, [key]: value })
}
</script>

<template>
  <div class="settings-page-heading settings-page-heading--compact">
    <div>
      <p class="settings-eyebrow">DESKTOP PET</p>
      <h1>桌宠与互动</h1>
      <p>调整桌面显示方式和互动反馈，修改会立即生效。</p>
    </div>
  </div>

  <div class="settings-group">
    <h2>窗口行为</h2>
    <SettingsRow title="始终置顶" description="让桌宠始终显示在其他窗口上方。">
      <input class="settings-switch" type="checkbox" :checked="settings.alwaysOnTop" @change="change(settings, 'alwaysOnTop', ($event.target as HTMLInputElement).checked)" />
    </SettingsRow>
    <SettingsRow title="鼠标穿透" description="开启后可直接点击桌面或下方的窗口。">
      <input class="settings-switch" type="checkbox" :checked="settings.clickThrough" @change="change(settings, 'clickThrough', ($event.target as HTMLInputElement).checked)" />
    </SettingsRow>
    <SettingsRow title="跟随鼠标" description="桌宠会向鼠标方向轻微转头。">
      <input class="settings-switch" type="checkbox" :checked="settings.followCursor" @change="change(settings, 'followCursor', ($event.target as HTMLInputElement).checked)" />
    </SettingsRow>
    <SettingsRow title="开机启动" description="登录 Windows 后自动启动 Qpets。">
      <input class="settings-switch" type="checkbox" :checked="settings.startWithWindows" @change="change(settings, 'startWithWindows', ($event.target as HTMLInputElement).checked)" />
    </SettingsRow>
  </div>

  <div class="settings-group">
    <h2>互动反馈</h2>
    <SettingsRow title="显示对话气泡" description="点击桌宠时显示简短的互动反馈。">
      <input class="settings-switch" type="checkbox" :checked="settings.showBubble" @change="change(settings, 'showBubble', ($event.target as HTMLInputElement).checked)" />
    </SettingsRow>
    <SettingsRow title="桌宠大小" :description="`${Math.round(settings.scale * 100)}%`">
      <input class="settings-range" type="range" min="0.6" max="1.6" step="0.05" :value="settings.scale" @input="change(settings, 'scale', Number(($event.target as HTMLInputElement).value))" />
    </SettingsRow>
    <SettingsRow title="闲置表演" :description="`约 ${settings.idleIntervalSeconds} 秒一次`">
      <input class="settings-range" type="range" min="20" max="60" step="5" :value="settings.idleIntervalSeconds" @input="change(settings, 'idleIntervalSeconds', Number(($event.target as HTMLInputElement).value))" />
    </SettingsRow>
  </div>
</template>
