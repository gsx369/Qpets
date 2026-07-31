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
    <SettingsRow title="始终置顶" description="让桌宠始终显示在其他窗口上方。" control-id="always-on-top" icon="pin">
      <input id="always-on-top" class="settings-switch" type="checkbox" aria-describedby="always-on-top-description" :checked="settings.alwaysOnTop" @change="change(settings, 'alwaysOnTop', ($event.target as HTMLInputElement).checked)" />
    </SettingsRow>
    <SettingsRow title="鼠标穿透" description="开启后可直接点击桌面或下方的窗口。" control-id="click-through" icon="pointer">
      <input id="click-through" class="settings-switch" type="checkbox" aria-describedby="click-through-description" :checked="settings.clickThrough" @change="change(settings, 'clickThrough', ($event.target as HTMLInputElement).checked)" />
    </SettingsRow>
    <SettingsRow title="跟随鼠标" description="桌宠会向鼠标方向轻微转头。" control-id="follow-cursor" icon="eye">
      <input id="follow-cursor" class="settings-switch" type="checkbox" aria-describedby="follow-cursor-description" :checked="settings.followCursor" @change="change(settings, 'followCursor', ($event.target as HTMLInputElement).checked)" />
    </SettingsRow>
    <SettingsRow title="开机启动" description="登录 Windows 后自动启动 Qpets。" control-id="start-with-windows" icon="power">
      <input id="start-with-windows" class="settings-switch" type="checkbox" aria-describedby="start-with-windows-description" :checked="settings.startWithWindows" @change="change(settings, 'startWithWindows', ($event.target as HTMLInputElement).checked)" />
    </SettingsRow>
  </div>

  <div class="settings-group">
    <h2>互动反馈</h2>
    <SettingsRow title="显示对话气泡" description="点击桌宠时显示简短的互动反馈。" control-id="show-bubble" icon="message">
      <input id="show-bubble" class="settings-switch" type="checkbox" aria-describedby="show-bubble-description" :checked="settings.showBubble" @change="change(settings, 'showBubble', ($event.target as HTMLInputElement).checked)" />
    </SettingsRow>
    <SettingsRow title="桌宠大小" description="调整角色在桌面的显示尺寸。" control-id="pet-scale" icon="scale">
      <div class="settings-range-control">
        <input id="pet-scale" class="settings-range" type="range" min="0.6" max="1.6" step="0.05" aria-describedby="pet-scale-description" :aria-valuetext="`${Math.round(settings.scale * 100)}%`" :value="settings.scale" @input="change(settings, 'scale', Number(($event.target as HTMLInputElement).value))" />
        <output for="pet-scale">{{ Math.round(settings.scale * 100) }}%</output>
      </div>
    </SettingsRow>
    <SettingsRow title="闲置表演" description="控制随机闲置动作的触发频率。" control-id="idle-interval" icon="clock">
      <div class="settings-range-control">
        <input id="idle-interval" class="settings-range" type="range" min="20" max="60" step="5" aria-describedby="idle-interval-description" :aria-valuetext="`约 ${settings.idleIntervalSeconds} 秒一次`" :value="settings.idleIntervalSeconds" @input="change(settings, 'idleIntervalSeconds', Number(($event.target as HTMLInputElement).value))" />
        <output for="idle-interval">{{ settings.idleIntervalSeconds }} 秒</output>
      </div>
    </SettingsRow>
  </div>
</template>
