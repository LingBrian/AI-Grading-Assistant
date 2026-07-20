<template>
  <el-dialog v-model="visible" fullscreen :modal="false" @close="handleClose">
    <template #header>
      <div class="dialog-header" data-tauri-drag-region>
        <span class="dialog-title">日志</span>
      </div>
    </template>
    <div class="toolbar">
      <el-select v-model="levelFilter" placeholder="全部级别" size="small" style="width: 120px" clearable>
        <el-option label="全部" value="" />
        <el-option label="DEBUG" value="DEBUG" />
        <el-option label="INFO" value="INFO" />
        <el-option label="WARN" value="WARN" />
        <el-option label="ERROR" value="ERROR" />
      </el-select>
      <el-button size="small" @click="refresh">刷新</el-button>
      <el-button size="small" type="danger" @click="clearLogs">清空</el-button>
    </div>
    <el-table :data="filteredLogs" style="width: 100%" size="small" v-loading="loading" max-height="calc(100vh - 160px)">
      <el-table-column prop="id" label="ID" width="55" />
      <el-table-column prop="level" label="级别" width="70">
        <template #default="{ row }">
          <el-tag :type="levelType(row.level)" size="small" effect="dark" style="width: 100%; text-align: center">
            {{ row.level }}
          </el-tag>
        </template>
      </el-table-column>
      <el-table-column prop="message" label="消息" min-width="300" show-overflow-tooltip />
      <el-table-column prop="source" label="来源" width="160" show-overflow-tooltip />
      <el-table-column prop="created_at" label="时间" width="155" />
    </el-table>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { useAppStore } from '@/stores/app'
import logger from '@/utils/logger'

interface LogEntry {
  id: number
  level: string
  message: string
  source: string
  created_at: string
}

const appStore = useAppStore()
const visible = ref(false)
const loading = ref(false)
const logs = ref<LogEntry[]>([])
const levelFilter = ref('')

const filteredLogs = computed(() => {
  if (!levelFilter.value) return logs.value
  return logs.value.filter(l => l.level === levelFilter.value)
})

function levelType(level: string) {
  switch (level) {
    case 'ERROR': return 'danger'
    case 'WARN': return 'warning'
    case 'INFO': return 'primary'
    default: return 'info'
  }
}

watch(() => appStore.isLogDialogOpen, (val) => {
  visible.value = val
  if (val) refresh()
})

async function refresh() {
  loading.value = true
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    logs.value = await invoke('log_list', { limit: 500 })
  } catch (e) {
    logger.error('refresh failed', 'LogDialog', e)
    logs.value = []
  } finally {
    loading.value = false
  }
}

async function clearLogs() {
  try {
    await ElMessageBox.confirm('确定清空所有日志？', '提示')
    const { invoke } = await import('@tauri-apps/api/core')
    await invoke('log_clear')
    ElMessage.success('日志已清空')
    refresh()
  } catch { /* cancelled */ }
}

function handleClose() {
  import('@tauri-apps/api/window').then(({ getCurrentWindow }) => getCurrentWindow().close())
}
</script>

<style scoped>
.toolbar {
  display: flex;
  gap: 8px;
  margin-bottom: 8px;
  align-items: center;
}
</style>
