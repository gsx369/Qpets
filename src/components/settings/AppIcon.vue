<script setup lang="ts">
import type { SettingsIconName } from '../../settings/icons'

withDefaults(defineProps<{
  name: SettingsIconName
  size?: number
}>(), {
  size: 18,
})
</script>

<template>
  <svg
    class="app-icon"
    :width="size"
    :height="size"
    viewBox="0 0 24 24"
    fill="none"
    stroke="currentColor"
    stroke-width="1.8"
    stroke-linecap="round"
    stroke-linejoin="round"
    aria-hidden="true"
    focusable="false"
  >
    <template v-if="name === 'library'">
      <rect x="3" y="3" width="7" height="7" rx="2" />
      <rect x="14" y="3" width="7" height="7" rx="2" />
      <rect x="3" y="14" width="7" height="7" rx="2" />
      <path d="M17.5 14v7M14 17.5h7" />
    </template>
    <template v-else-if="name === 'sliders'">
      <path d="M4 6h6M14 6h6M4 12h10M18 12h2M4 18h2M10 18h10" />
      <circle cx="12" cy="6" r="2" /><circle cx="16" cy="12" r="2" /><circle cx="8" cy="18" r="2" />
    </template>
    <template v-else-if="name === 'info'">
      <circle cx="12" cy="12" r="9" /><path d="M12 11v5" /><path d="M12 8h.01" />
    </template>
    <template v-else-if="name === 'image-plus'">
      <rect x="3" y="5" width="14" height="14" rx="2" /><circle cx="8" cy="10" r="1.5" />
      <path d="m4 17 4-4 3 3 2-2 4 4M20 4v6M17 7h6" />
    </template>
    <template v-else-if="name === 'package'">
      <path d="m12 3 8 4.5v9L12 21l-8-4.5v-9L12 3Z" /><path d="m4.5 7.8 7.5 4.1 7.5-4.1M12 12v9M8 5.2l8 4.5" />
    </template>
    <template v-else-if="name === 'edit'">
      <path d="M4 20h4l11-11a2.8 2.8 0 0 0-4-4L4 16v4Z" /><path d="m13.5 6.5 4 4" />
    </template>
    <template v-else-if="name === 'trash'">
      <path d="M4 7h16M9 7V4h6v3M7 7l1 13h8l1-13M10 11v5M14 11v5" />
    </template>
    <template v-else-if="name === 'external'">
      <path d="M14 4h6v6M20 4l-9 9" /><path d="M18 13v6a1 1 0 0 1-1 1H5a1 1 0 0 1-1-1V7a1 1 0 0 1 1-1h6" />
    </template>
    <template v-else-if="name === 'plus'"><path d="M12 5v14M5 12h14" /></template>
    <template v-else-if="name === 'check'"><path d="m5 12 4 4L19 6" /></template>
    <template v-else-if="name === 'alert'">
      <path d="M12 3 2.8 19h18.4L12 3Z" /><path d="M12 9v4M12 16h.01" />
    </template>
    <template v-else-if="name === 'close'"><path d="m6 6 12 12M18 6 6 18" /></template>
    <template v-else-if="name === 'reset'">
      <path d="M4 4v6h6" /><path d="M5.5 15a8 8 0 1 0 .7-7.6L4 10" />
    </template>
    <template v-else-if="name === 'pin'">
      <path d="M8 4h8l-1 6 3 3H6l3-3-1-6ZM12 13v8" />
    </template>
    <template v-else-if="name === 'pointer'">
      <path d="m5 3 13 9-6 1.5L9 19 5 3Z" />
    </template>
    <template v-else-if="name === 'eye'">
      <path d="M2.5 12s3.5-6 9.5-6 9.5 6 9.5 6-3.5 6-9.5 6-9.5-6-9.5-6Z" /><circle cx="12" cy="12" r="2.5" />
    </template>
    <template v-else-if="name === 'power'">
      <path d="M12 3v8" /><path d="M7.3 5.8a8 8 0 1 0 9.4 0" />
    </template>
    <template v-else-if="name === 'message'">
      <path d="M4 5h16v11H9l-5 4V5Z" /><path d="M8 9h8M8 12h5" />
    </template>
    <template v-else-if="name === 'scale'">
      <path d="M8 3H3v5M16 3h5v5M8 21H3v-5M16 21h5v-5" /><path d="m3 8 5-5M21 8l-5-5M3 16l5 5M21 16l-5 5" />
    </template>
    <template v-else-if="name === 'clock'">
      <circle cx="12" cy="12" r="9" /><path d="M12 7v5l3 2" />
    </template>
    <template v-else>
      <path d="m12 3 1.2 4.2L17 9l-3.8 1.8L12 15l-1.2-4.2L7 9l3.8-1.8L12 3Z" />
      <path d="m18.5 15 .7 2.3 2.3.7-2.3.7-.7 2.3-.7-2.3-2.3-.7 2.3-.7.7-2.3Z" />
    </template>
  </svg>
</template>
