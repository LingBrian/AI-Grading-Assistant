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
          <div style="display:flex;gap:8px;width:100%;flex-wrap:wrap">
            <el-select v-model="activePreset" placeholder="选择预设" size="small" style="width:120px" @change="applyPreset" clearable>
              <el-option v-for="(key, name) in form.apiKeyPresets" :key="name" :label="name" :value="name" />
            </el-select>
            <el-input v-model="form.apiKey" type="password" show-password placeholder="输入或选择预设 API Key" style="flex:1;min-width:150px" />
            <el-button size="small" @click="savePreset" :disabled="!form.apiKey">保存预设</el-button>
            <el-button size="small" @click="deletePreset" :disabled="!activePreset" type="danger">删除</el-button>
          </div>
        </el-form-item>

        <el-form-item label="Base URL">
          <div style="display:flex;gap:8px;width:100%">
            <el-select v-model="form.baseUrl" filterable allow-create default-first-option placeholder="选择或输入 API 地址" style="flex:1">
              <el-option-group label="预设服务商">
                <el-option v-for="p in presetProviders" :key="p.url" :label="p.label" :value="p.url" />
              </el-option-group>
            </el-select>
          </div>
        </el-form-item>

        <el-form-item label="模型">
          <div style="display:flex;gap:8px;width:100%">
            <el-select v-model="form.model" filterable allow-create default-first-option placeholder="输入或选择模型" style="flex:1">
              <el-option v-for="m in fetchedModels" :key="m" :label="m" :value="m" />
            </el-select>
            <el-button size="small" @click="fetchModels" :loading="fetchingModels">获取模型</el-button>
          </div>
          <div style="margin-top: 6px; display: flex; flex-wrap: wrap; gap: 6px;">
            <el-tag
              v-for="m in popularModels"
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
import { ElMessage, ElMessageBox } from 'element-plus'
import { useAppStore } from '@/stores/app'
import { useSettingsStore } from '@/stores/settings'
import logger from '@/utils/logger'

const appStore = useAppStore()
const settingsStore = useSettingsStore()

const visible = ref(false)
const fetchedModels = ref<string[]>([])
const fetchingModels = ref(false)
const activePreset = ref('')

const presetProviders = [
  { label: 'DeepSeek', url: 'https://api.deepseek.com/v1' },
  { label: 'Kimi (Moonshot)', url: 'https://api.moonshot.cn/v1' },
  { label: 'SiliconFlow', url: 'https://api.siliconflow.cn/v1' },
  { label: 'Qwen (DashScope)', url: 'https://dashscope.aliyuncs.com/compatible-mode/v1' },
]

const popularModels = [
  'Qwen/Qwen2.5-32B-Instruct',
  'Qwen/Qwen3.6-35B-A3B',
  'gpt-4o',
  'claude-3-5-sonnet-20241022',
  'gemini-2.0-flash-exp',
  'deepseek-vl2'
]

watch(() => appStore.isSettingDialogOpen, (val) => {
  visible.value = val
  if (val) {
    form.value = { ...settingsStore.settings }
  }
})

const form = ref({ ...settingsStore.settings })

async function fetchModels() {
  if (!form.value.baseUrl || !form.value.apiKey) {
    ElMessage.warning('请先填写 API Key 和 Base URL')
    return
  }
  fetchingModels.value = true
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    const models: string[] = await invoke('ai_fetch_models', {
      request: { apiKey: form.value.apiKey, baseUrl: form.value.baseUrl }
    })
    fetchedModels.value = models
    if (models.length > 0) {
      ElMessage.success(`获取到 ${models.length} 个模型`)
    }
  } catch (e) {
    const msg = String((e as any)?.message ?? e)
    ElMessage.error('获取模型列表失败：' + msg)
  } finally {
    fetchingModels.value = false
  }
}

function applyPreset(name: string) {
  if (name && form.value.apiKeyPresets[name]) {
    form.value.apiKey = form.value.apiKeyPresets[name]
  }
}

async function savePreset() {
  try {
    const { value } = await ElMessageBox.prompt('请输入预设名称', '保存 API Key', { inputPattern: /\S+/, inputErrorMessage: '名称不能为空' })
    form.value.apiKeyPresets = { ...form.value.apiKeyPresets, [value]: form.value.apiKey }
    activePreset.value = value
    ElMessage.success(`预设 "${value}" 已保存`)
  } catch { /* cancelled */ }
}

function deletePreset() {
  if (!activePreset.value) return
  const name = activePreset.value
  const { [name]: _, ...rest } = form.value.apiKeyPresets
  form.value.apiKeyPresets = rest
  activePreset.value = ''
  ElMessage.success(`预设 "${name}" 已删除`)
}

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
    const msg = String((err as any)?.message ?? err)
    logger.error(msg, 'SettingDialog')
    ElMessage.error('保存失败：' + msg)
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
