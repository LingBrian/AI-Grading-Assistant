<template>
  <template v-if="dialogType">
    <SettingDialog v-if="dialogType === 'settings'" />
    <HistoryDialog v-if="dialogType === 'history'" />
    <TemplateDialog v-if="dialogType === 'templates'" />
    <RubricEditor v-if="dialogType === 'rubric'" />
    <ResultDrawer v-if="dialogType === 'result'" />
  </template>
  <template v-else>
    <router-view />
  </template>
</template>

<script setup lang="ts">
import { nextTick, onMounted } from 'vue'
import { useAppStore } from '@/stores/app'
import { useSettingsStore } from '@/stores/settings'
import { useGradingStore } from '@/stores/grading'
import { LogicalSize, getCurrentWindow } from '@tauri-apps/api/window'
import ResultDrawer from '@/components/ResultDrawer.vue'
import SettingDialog from '@/components/SettingDialog.vue'
import HistoryDialog from '@/pages/HistoryDialog.vue'
import TemplateDialog from '@/pages/TemplateDialog.vue'
import RubricEditor from '@/components/RubricEditor.vue'

const appStore = useAppStore()
const settingsStore = useSettingsStore()
const gradingStore = useGradingStore()

const dialogType = new URLSearchParams(window.location.search).get('dialog')
const isScreenshot = window.location.hash.startsWith('#/screenshot')

const winClass = isScreenshot ? 'win-screenshot' : dialogType ? 'win-dialog' : 'win-main'
document.documentElement.dataset.window = winClass

async function resizeToContent() {
  await nextTick()
  await nextTick()
  const el = document.querySelector('.el-dialog') || document.querySelector('.drawer-panel') || document.querySelector('.win-dialog-body')
  if (!el) return
  const rect = el.getBoundingClientRect()
  try {
    const w = Math.ceil(rect.width) + 40
    const h = Math.ceil(rect.height) + 80
    await getCurrentWindow().setSize(new LogicalSize(Math.max(300, w), Math.max(200, h)))
  } catch {}
}

onMounted(async () => {
  if (dialogType === 'result') {
    const { invoke } = await import('@tauri-apps/api/core')
    const data: any = await invoke('grading_get_data')
    if (data) {
      if (data.image) gradingStore.setImage(data.image)
      if (data.questions) gradingStore.setQuestions(data.questions)
      if (data.result) gradingStore.setResult(data.result)
      if (data.currentQuestionIndex !== undefined) gradingStore.setCurrentQuestionIndex(data.currentQuestionIndex)
    }
    appStore.openResultDrawer()
    await resizeToContent()
  } else if (dialogType) {
    if (dialogType === 'settings') await settingsStore.loadSettings()
    const actions: Record<string, () => void> = {
      settings: () => appStore.openSettingDialog(),
      history: () => appStore.openHistoryDialog(),
      templates: () => appStore.openTemplateDialog(),
      rubric: () => appStore.openRubricEditor()
    }
    actions[dialogType]?.()
    setTimeout(resizeToContent, 200)
  } else {
    await settingsStore.loadSettings()
  }
})
</script>
