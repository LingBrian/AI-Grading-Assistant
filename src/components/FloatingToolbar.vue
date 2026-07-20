<template>
  <div ref="toolbarRef" class="floating-toolbar" :class="{ 'is-hidden': !appStore.isToolbarVisible, 'has-notification': !!notification }" @mousedown="onDragStart">
    <div class="toolbar-buttons">
      <el-button class="toolbar-btn" @click="startScreenshot" :loading="appStore.isGrading">
        <span class="btn-icon">📷</span>
        <span class="btn-label">{{ appStore.isGrading ? '阅卷中...' : '截图阅卷' }}</span>
      </el-button>
      <el-button class="toolbar-btn" @click="openRubric">
        <span class="btn-icon">📋</span>
        <span class="btn-label">评分标准</span>
      </el-button>
      <template v-if="!showMenu">
        <el-button class="toolbar-btn" @click="showMenu = true">
          <span class="btn-icon">📎</span>
          <span class="btn-label">其它</span>
        </el-button>
      </template>
      <template v-else>
        <el-button class="toolbar-btn sub-btn" @click="showMenu = false; selectImage()">
          <span class="btn-icon">📁</span>
          <span class="btn-label">图片阅卷</span>
        </el-button>
        <el-button class="toolbar-btn sub-btn" @click="showMenu = false; openTemplates()">
          <span class="btn-icon">📚</span>
          <span class="btn-label">模板</span>
        </el-button>
        <el-button class="toolbar-btn sub-btn" @click="showMenu = false; openHistory()">
          <span class="btn-icon">📜</span>
          <span class="btn-label">历史</span>
        </el-button>
        <el-button class="toolbar-btn sub-btn" @click="showMenu = false; openLogs()">
          <span class="btn-icon">📋</span>
          <span class="btn-label">日志</span>
        </el-button>
        <el-button class="toolbar-btn collapse-btn" @click="showMenu = false">
          <span class="btn-icon">↩</span>
          <span class="btn-label">收起</span>
        </el-button>
      </template>
      <el-button class="toolbar-btn" @click="openSettings">
        <span class="btn-icon">⚙</span>
        <span class="btn-label">设置</span>
      </el-button>
    </div>
    <div class="status-bar">{{ statusText }}</div>
    <div v-if="notification" :class="['notification-bar', `notification-${notificationType}`]">{{ notification }}</div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, nextTick, watch, onMounted, onUnmounted } from 'vue'
import { ElMessage } from 'element-plus'
import { useAppStore } from '@/stores/app'
import { useGradingStore } from '@/stores/grading'
import { useSettingsStore } from '@/stores/settings'
import { addWatermark } from '@/utils/image'
import { getCurrentWindow, LogicalSize } from '@tauri-apps/api/window'
import logger from '@/utils/logger'

const appStore = useAppStore()
const gradingStore = useGradingStore()
const settingsStore = useSettingsStore()

const showMenu = ref(false)
const toolbarRef = ref<HTMLElement | null>(null)
const notification = ref('')
const notificationType = ref<'success' | 'error' | 'warning'>('success')
let notificationTimer: ReturnType<typeof setTimeout> | null = null

function showNotification(msg: string, type: 'success' | 'error' | 'warning' = 'success', duration = 4000) {
  notification.value = msg
  notificationType.value = type
  if (notificationTimer) clearTimeout(notificationTimer)
  notificationTimer = setTimeout(() => { notification.value = '' }, duration)
}

const statusText = computed(() => {
  const questions = gradingStore.questions
  const idx = gradingStore.currentQuestionIndex
  return questions?.length ? `第 ${idx + 1}/${questions.length} 题` : ''
})

async function resizeWindow() {
  await nextTick()
  if (toolbarRef.value) {
    try {
      const w = toolbarRef.value.scrollWidth + 2
      const h = toolbarRef.value.scrollHeight + 2
      await getCurrentWindow().setSize(new LogicalSize(Math.max(200, Math.min(w, 1200)), h))
    } catch {}
  }
}
watch(showMenu, resizeWindow)
watch(notification, resizeWindow)

let unlistenScreenshot: (() => void) | null = null
let unlistenRubric: (() => void) | null = null

onMounted(async () => {
  resizeWindow()
  try {
    const { listen } = await import('@tauri-apps/api/event')
    unlistenScreenshot = await listen<string>('screenshot-completed', async (event) => {
      if (!appStore.isGrading) {
        try {
          gradingStore.setImage(await addWatermark(event.payload, '学生答案'))
          await startGrading()
        } catch (err) {
          const msg = String((err as any)?.message ?? err)
          logger.error(msg, 'screenshot-completed')
          ElMessage.error('水印处理失败：' + msg)
        }
      }
    })
    unlistenRubric = await listen('rubric-updated', async () => {
      const { invoke } = await import('@tauri-apps/api/core')
      const data: any = await invoke('rubric_get_data')
      if (data?.questions?.length) {
        gradingStore.setQuestions(JSON.parse(JSON.stringify(data.questions)))
        ElMessage.success(`已应用模板（${data.questions.length} 题）`)
      }
    })
  } catch (e) { logger.error('listen rubric-updated failed', 'FloatingToolbar', e) }
})

onUnmounted(() => {
  unlistenScreenshot?.()
  unlistenRubric?.()
})

async function startScreenshot() {
  const { invoke } = await import('@tauri-apps/api/core')
  try {
    await invoke('screenshot_start')
  } catch (err) {
    const msg = String((err as any)?.message ?? err)
    logger.error(msg, 'startScreenshot')
    ElMessage.error('截图失败：' + msg)
  }
}

async function selectImage() {
  const { invoke } = await import('@tauri-apps/api/core')
  const data: any = await invoke('dialog_open_image')
  if (data) {
    try { gradingStore.setImage(await addWatermark(data.base64, '学生答案')) } catch (err) {
      const msg = String((err as any)?.message ?? err)
      logger.error(msg, 'selectImage')
      ElMessage.error('水印处理失败：' + msg); return
    }
    await startGrading()
  }
}

async function gradeQuestion(questionIdx: number) {
  const { invoke } = await import('@tauri-apps/api/core')
  const questions = gradingStore.questions
  if (!questions.length) { ElMessage.warning('请先在评分标准中添加题目'); return null }
  const q = questions[Math.min(questionIdx, questions.length - 1)]
  const result: any = await invoke('ai_grade', {
    request: {
      imageBase64: gradingStore.currentImage,
      rubric: q.rubric || `请根据答案内容评分，满分${q.maxScore}分`,
      maxScore: q.maxScore,
      referenceAnswer: q.referenceAnswer || '',
      referenceAnswerImage: q.referenceAnswerImage || '',
      questionTitle: q.questionTitle || '',
      apiConfig: {
        apiKey: settingsStore.settings.apiKey,
        baseUrl: settingsStore.settings.baseUrl,
        model: settingsStore.settings.model
      }
    }
  })
  return result
}

async function startGrading() {
  await settingsStore.loadSettings()
  if (!gradingStore.currentImage) { ElMessage.info('请先截图或选择图片'); return }
  if (!settingsStore.settings.apiKey) {
    ElMessage.warning('请先在设置中配置 API Key')
    const { invoke } = await import('@tauri-apps/api/core')
    await invoke('dialog_open', { dialogType: 'settings' })
    return
  }
  const { invoke } = await import('@tauri-apps/api/core')
  const rubricData: any = await invoke('rubric_get_data')
  const questions = rubricData?.questions?.length
    ? JSON.parse(JSON.stringify(rubricData.questions))
    : [{ questionTitle: '', maxScore: 100, referenceAnswer: '', rubric: '' }]
  const nextIdx = Math.min(await invoke('rubric_get_next_index') as number ?? 0, questions.length - 1)
  gradingStore.setQuestions(questions)
  gradingStore.setCurrentQuestionIndex(nextIdx)
  appStore.isGrading = true
  try {
    const result = await gradeQuestion(nextIdx)
    if (!result) return
    gradingStore.setResult(result)
    showNotification(`得分：${result.score} 分（${(result.confidence * 100).toFixed(0)}% 置信度）`, 'success')
    await invoke('grading_store_data', {
      data: { result, image: gradingStore.currentImage, questions, currentQuestionIndex: nextIdx }
    })
    await invoke('dialog_open', { dialogType: 'result' })
    gradingStore.reset()
  } catch (err) {
    let msg = String((err as any)?.message ?? err)
    msg = msg.replace(/Error invoking '\w+':\s*/i, '')
    if (/does not support image|not support.*image|image.*input|Cannot read|image\.png/i.test(msg)) {
      msg = '当前模型不支持图片输入，请在设置中更换为支持视觉的模型'
    } else if (/does not support|model.*capable|image.*not/i.test(msg)) {
      msg = '当前模型不支持图片输入，请在设置中更换为支持视觉的模型'
    }
    logger.error(msg, 'startGrading')
    ElMessage.error(msg)
    showNotification(msg, 'error', 6000)
  } finally { appStore.isGrading = false }
}

function openRubric() {
  import('@tauri-apps/api/core').then(({ invoke }) => invoke('dialog_open', { dialogType: 'rubric' }))
}
async function openTemplates() {
  const { invoke } = await import('@tauri-apps/api/core')
  await invoke('dialog_open', { dialogType: 'templates' })
}
function openHistory() {
  import('@tauri-apps/api/core').then(({ invoke }) => invoke('dialog_open', { dialogType: 'history' }))
}
function openLogs() {
  import('@tauri-apps/api/core').then(({ invoke }) => invoke('dialog_open', { dialogType: 'logs' }))
}
function openSettings() {
  import('@tauri-apps/api/core').then(({ invoke }) => invoke('dialog_open', { dialogType: 'settings' }))
}

function onDragStart(e: MouseEvent) {
  if ((e.target as HTMLElement)?.closest('.toolbar-btn')) return
  e.preventDefault()
  getCurrentWindow().startDragging()
}
</script>

<style scoped>
.floating-toolbar { position: fixed; top: 0; left: 0; width: fit-content; min-width: 200px; z-index: 99999; user-select: none; background: rgba(30,30,30,0.85); backdrop-filter: blur(12px); border-radius: 12px; box-shadow: 0 4px 24px rgba(0,0,0,0.3); }
.toolbar-buttons { display: flex; gap: 6px; padding: 8px 14px 0; justify-content: center; align-items: center; }
.status-bar { padding: 0 14px 6px; font-size: 11px; font-weight: 700; color: rgba(255,255,255,0.6); white-space: nowrap; text-align: center; }
.sub-btn { padding: 4px 8px !important; }
.collapse-btn { padding: 4px 6px !important; color: rgba(255,255,255,0.5) !important; }
.collapse-btn:hover { color: rgba(255,255,255,0.9) !important; }
.toolbar-btn { border: none !important; background: transparent !important; color: rgba(255,255,255,0.85) !important; height: 46px !important; padding: 4px 14px !important; display: flex; flex-direction: column; align-items: center; gap: 1px; transition: all 0.2s; cursor: pointer; }
.toolbar-btn:hover { background: rgba(255,255,255,0.12) !important; color: #fff !important; }
.btn-icon { font-size: 16px; line-height: 1; }
.btn-label { font-size: 10px; line-height: 1; white-space: nowrap; }
.is-hidden { opacity: 0; pointer-events: none; }
.notification-bar { padding: 4px 14px 8px; font-size: 12px; font-weight: 600; white-space: nowrap; text-align: center; border-radius: 0 0 12px 12px; }
.notification-success { color: #67c23a; background: rgba(103,194,58,0.1); }
.notification-error { color: #f56c6c; background: rgba(245,108,108,0.1); }
.notification-warning { color: #e6a23c; background: rgba(230,162,60,0.1); }
</style>
