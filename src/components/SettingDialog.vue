<template>
  <el-dialog v-model="visible" fullscreen :modal="false" :close-on-click-modal="false" @close="handleClose">
    <template #header>
      <div class="dialog-header" data-tauri-drag-region>
        <span class="dialog-title">设置</span>
      </div>
    </template>

    <div class="settings-body">
      <el-form :model="form" label-width="100px" size="small">
        <el-form-item label="API Key">
          <el-input v-model="form.apiKey" type="password" show-password placeholder="请输入 API Key" />
        </el-form-item>

        <el-form-item label="Base URL">
          <el-input v-model="form.baseUrl" placeholder="https://api.openai.com/v1" />
        </el-form-item>

        <el-form-item label="模型">
          <el-input v-model="form.model" placeholder="手动输入模型名称" />
          <div style="margin-top: 6px; display: flex; flex-wrap: wrap; gap: 6px;">
            <el-tag
              v-for="m in visionModels"
              :key="m"
              size="small"
              style="cursor:pointer"
              @click="form.model = m"
            >{{ m }}</el-tag>
          </div>
        </el-form-item>

        <el-form-item label="快捷键">
          <el-input v-model="form.shortcut" placeholder="Alt+Q" />
        </el-form-item>

        <el-form-item label="超时(秒)">
          <el-input-number v-model="form.timeout" :min="5" :max="300" />
        </el-form-item>

        <el-form-item label="主题">
          <el-select v-model="form.theme" style="width: 100%">
            <el-option label="浅色" value="light" />
            <el-option label="深色" value="dark" />
          </el-select>
        </el-form-item>

        <el-form-item label="字体大小">
          <el-input-number v-model="form.fontSize" :min="12" :max="20" />
        </el-form-item>

        <el-form-item label="代理">
          <el-input v-model="form.proxy" placeholder="可选，如 http://127.0.0.1:7890" />
        </el-form-item>

        <el-form-item label="截图保存路径">
          <div style="display:flex;gap:8px;width:100%">
            <el-input v-model="form.screenshotSavePath" placeholder="留空不自动保存截图" />
            <el-button @click="selectDir">选择</el-button>
          </div>
        </el-form-item>
      </el-form>
    </div>

    <template #footer>
      <el-button size="small" @click="handleClose">取消</el-button>
      <el-button type="primary" size="small" @click="handleSave">保存</el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { useAppStore } from '@/stores/app'
import { useSettingsStore } from '@/stores/settings'

const visionModels = [
  'Qwen/Qwen2.5-32B-Instruct',
  'Qwen/Qwen3.6-35B-A3B',
  'gpt-4o',
  'claude-3-5-sonnet-20241022',
  'gemini-2.0-flash-exp',
  'deepseek-vl2'
]

const appStore = useAppStore()
const settingsStore = useSettingsStore()

const visible = ref(false)

watch(() => appStore.isSettingDialogOpen, (val) => {
  visible.value = val
  if (val) {
    form.value = { ...settingsStore.settings }
  }
})

const form = ref({ ...settingsStore.settings })

function handleClose() {
  import('@tauri-apps/api/window').then(({ getCurrentWindow }) => getCurrentWindow().close())
}

async function selectDir() {
  const { open } = await import('@tauri-apps/plugin-dialog')
  const dir = await open({ directory: true })
  if (dir) form.value.screenshotSavePath = dir
}

async function handleSave() {
  try {
    await settingsStore.saveSettings(JSON.parse(JSON.stringify(form.value)))
    const { getCurrentWindow } = await import('@tauri-apps/api/window')
    await getCurrentWindow().close()
  } catch (err) {
    ElMessage.error('保存失败：' + (err as Error).message)
  }
}
</script>

<style scoped>
.dialog-header {
  display: flex; align-items: center; padding: 0;
}
.dialog-title {
  font-size: 15px; font-weight: 600;
}
.settings-body {
  flex: 1; overflow-y: auto; min-height: 0;
}
:deep(.el-dialog__body) {
  display: flex; flex-direction: column; flex: 1; min-height: 0; padding: 20px;
}
:deep(.el-dialog__footer) {
  flex-shrink: 0;
}
</style>
