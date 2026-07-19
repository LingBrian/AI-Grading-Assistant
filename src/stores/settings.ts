import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { AppSettings } from '@/types'

export const useSettingsStore = defineStore('settings', () => {
  const settings = ref<AppSettings>({
    apiKey: '',
    baseUrl: 'https://api.siliconflow.cn/v1',
    model: '',
    shortcut: 'Alt+Q',
    theme: 'light',
    fontSize: 14,
    proxy: '',
    timeout: 120,
    screenshotSavePath: ''
  })

  async function loadSettings() {
    try {
      const { invoke } = await import('@tauri-apps/api/core')
      const result: AppSettings = await invoke('settings_get')
      if (result) {
        settings.value = { ...settings.value, ...result }
      }
    } catch {}
  }

  async function saveSettings(partial: Partial<AppSettings>) {
    Object.assign(settings.value, partial)
    const { invoke } = await import('@tauri-apps/api/core')
    await invoke('settings_set', { settings: settings.value })
  }

  return { settings, loadSettings, saveSettings }
})
