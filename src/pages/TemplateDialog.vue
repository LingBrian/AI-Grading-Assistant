<template>
  <el-dialog v-model="visible" fullscreen :modal="false" @close="handleClose">
    <template #header>
      <div class="dialog-header" data-tauri-drag-region>
        <span class="dialog-title">评分模板管理</span>
      </div>
    </template>

    <div class="template-body">
      <el-table :data="templates" size="small" style="width: 100%" height="100%">
        <el-table-column prop="id" label="ID" width="50" />
        <el-table-column prop="name" label="考试名称" min-width="130" />
        <el-table-column prop="course" label="课程" min-width="100" />
        <el-table-column label="题目数" width="70">
          <template #default="{ row }">{{ row.questions?.length || 0 }}</template>
        </el-table-column>
        <el-table-column prop="created_at" label="创建时间" width="150" />
        <el-table-column label="操作" width="120" fixed="right">
          <template #default="{ row }">
            <el-button text size="small" :disabled="!row.questions?.length" @click="selectTemplate(row)">使用</el-button>
            <el-button text size="small" type="danger" @click="deleteTemplate(row)">删除</el-button>
          </template>
        </el-table-column>
      </el-table>
    </div>

    <template #footer>
      <div class="footer-left">
        <el-button size="small" @click="exportTemplates">导出</el-button>
        <el-button size="small" @click="importTemplates">导入</el-button>
      </div>
      <el-button size="small" @click="handleClose">关闭</el-button>
    </template>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { useAppStore } from '@/stores/app'
import type { TemplateItem } from '@/types'

const appStore = useAppStore()
const visible = ref(false)
const templates = ref<TemplateItem[]>([])

watch(() => appStore.isTemplateDialogOpen, (val) => {
  visible.value = val
  if (val) loadTemplates()
})

async function loadTemplates() {
  const { invoke } = await import('@tauri-apps/api/core')
  templates.value = await invoke('template_list')
}

async function selectTemplate(tpl: TemplateItem) {
  if (!tpl.questions?.length) {
    ElMessage.warning('该模板没有题目数据')
    return
  }
  const { invoke } = await import('@tauri-apps/api/core')
  const ok: boolean = await invoke('template_apply', { id: tpl.id })
  if (ok) {
    handleClose()
  } else {
    ElMessage.error('应用模板失败')
  }
}

async function exportTemplates() {
  const { invoke } = await import('@tauri-apps/api/core')
  const ok: boolean = await invoke('template_export_json')
  if (ok) {
    ElMessage.success('模板已导出')
  } else if (ok === false) {
    ElMessage.warning('导出失败或已取消')
  }
}

async function importTemplates() {
  const { invoke } = await import('@tauri-apps/api/core')
  const result: any = await invoke('template_import_json')
  if (result) {
    ElMessage.success(`已导入 ${result.count} 个模板`)
    loadTemplates()
  }
}

async function deleteTemplate(tpl: TemplateItem) {
  const { invoke } = await import('@tauri-apps/api/core')
  const ok: boolean = await invoke('template_delete', { id: tpl.id })
  if (ok) {
    ElMessage.success('已删除')
    loadTemplates()
  }
}

function handleClose() {
  import('@tauri-apps/api/window').then(({ getCurrentWindow }) => getCurrentWindow().close())
}
</script>

<style scoped>
.dialog-header {
  display: flex; align-items: center; padding: 0;
}
.dialog-title {
  font-size: 15px; font-weight: 600;
}
.template-body {
  flex: 1; min-height: 0;
}
:deep(.el-dialog__body) {
  display: flex; flex-direction: column; flex: 1; min-height: 0; padding: 20px;
}
:deep(.el-dialog__footer) {
  display: flex; justify-content: space-between; align-items: center; flex-shrink: 0;
}
.footer-left {
  display: flex; gap: 8px;
}
</style>
