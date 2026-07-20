<template>
  <el-dialog v-model="visible" fullscreen :modal="false" @close="handleClose">
    <template #header>
      <div class="dialog-header" data-tauri-drag-region>
        <span class="dialog-title">评分标准</span>
      </div>
    </template>

    <div class="rubric-layout">
      <div class="tab-list">
        <div
          v-for="(q, idx) in questions"
          :key="idx"
          class="tab-item"
          :class="{ active: idx === activeIndex }"
          @click="activeIndex = idx"
        >
          <span>题目 {{ idx + 1 }}</span>
          <el-button text size="small" @click.stop="removeQuestion(idx)" v-if="questions.length > 1">✕</el-button>
        </div>
        <el-button size="small" @click="addQuestion">+ 添加题目</el-button>
      </div>

      <div class="tab-content" v-if="questions[activeIndex]">
        <el-form :model="questions[activeIndex]" size="small" label-width="80px">
          <el-form-item label="题目">
            <el-input v-model="questions[activeIndex].questionTitle" type="textarea" :rows="2" placeholder="输入题目内容" />
          </el-form-item>
          <el-form-item label="满分">
            <el-input-number v-model="questions[activeIndex].maxScore" :min="1" :max="1000" />
          </el-form-item>
          <el-form-item label="参考答案">
            <el-input v-model="questions[activeIndex].referenceAnswer" type="textarea" :rows="3" placeholder="输入参考答案（可选）" />
            <div class="ref-image-section">
              <template v-if="questions[activeIndex].referenceAnswerImage">
                <div class="ref-image-preview">
                  <img :src="questions[activeIndex].referenceAnswerImage" />
                  <el-button size="small" type="danger" text @click="removeRefImage(activeIndex)">删除图片</el-button>
                </div>
              </template>
              <el-button size="small" @click="uploadRefImage(activeIndex)">上传参考答案图片</el-button>
            </div>
          </el-form-item>
          <el-form-item label="评分标准">
            <el-input v-model="questions[activeIndex].rubric" type="textarea" :rows="6" placeholder="输入评分标准" />
          </el-form-item>
        </el-form>
      </div>
    </div>

    <template #footer>
      <div class="footer-left">
        <el-button size="small" @click="handleImport">导入</el-button>
        <el-button size="small" @click="handleExport">导出</el-button>
        <el-button size="small" @click="showTemplateForm = !showTemplateForm">
          {{ showTemplateForm ? '取消' : '保存为模板' }}
        </el-button>
      </div>
      <div class="footer-right">
        <el-button size="small" @click="handleClose">取消</el-button>
        <el-button type="primary" size="small" @click="handleSave">应用</el-button>
      </div>
    </template>

    <div v-if="showTemplateForm" class="template-form">
      <el-form inline size="small">
        <el-form-item label="模板名称">
          <el-input v-model="templateName" placeholder="如：期末考试" />
        </el-form-item>
        <el-form-item label="课程">
          <el-input v-model="templateCourse" placeholder="如：Java" />
        </el-form-item>
        <el-form-item>
          <el-button type="success" size="small" @click="saveAsTemplate">保存</el-button>
        </el-form-item>
      </el-form>
    </div>
  </el-dialog>
</template>

<script setup lang="ts">
import { ref, watch } from 'vue'
import { ElMessage } from 'element-plus'
import { useAppStore } from '@/stores/app'
import { useGradingStore } from '@/stores/grading'
import type { QuestionItem } from '@/types'
import { addWatermark } from '@/utils/image'
import logger from '@/utils/logger'

const appStore = useAppStore()
const gradingStore = useGradingStore()
const visible = ref(false)
const activeIndex = ref(0)
const questions = ref<QuestionItem[]>([])
const showTemplateForm = ref(false)
const templateName = ref('')
const templateCourse = ref('')

function emptyQuestion(): QuestionItem {
  return { questionTitle: '', maxScore: 100, referenceAnswer: '', rubric: '' }
}

watch(() => appStore.isRubricEditorOpen, async (val) => {
  visible.value = val
  if (val) {
    const { invoke } = await import('@tauri-apps/api/core')
    const data: any = await invoke('rubric_get_data')
    if (data?.questions?.length) {
      questions.value = JSON.parse(JSON.stringify(data.questions))
      activeIndex.value = data.currentIndex ?? 0
    } else {
      questions.value = [emptyQuestion()]
      activeIndex.value = 0
    }
  }
})

function addQuestion() {
  questions.value.push(emptyQuestion())
  activeIndex.value = questions.value.length - 1
}

function removeQuestion(idx: number) {
  questions.value.splice(idx, 1)
  if (activeIndex.value >= questions.value.length) {
    activeIndex.value = questions.value.length - 1
  }
}

async function handleSave() {
  const { invoke } = await import('@tauri-apps/api/core')
  try {
    gradingStore.setQuestions(JSON.parse(JSON.stringify(questions.value)))
    await invoke('rubric_store_data', {
      data: {
        questions: JSON.parse(JSON.stringify(questions.value)),
        currentIndex: 0
      }
    })
    await invoke('rubric_reset_next_index')
    ElMessage.success('评分标准已保存')
  } catch (err) {
    const msg = String((err as any)?.message ?? err)
    logger.error(msg, 'RubricEditor.handleSave')
    ElMessage.error('保存失败：' + msg)
  }
}

async function handleExport() {
  const { invoke } = await import('@tauri-apps/api/core')
  try {
    await invoke('rubric_export_json')
    ElMessage.success('导出成功')
  } catch (err) {
    const msg = String((err as any)?.message ?? err)
    logger.error(msg, 'RubricEditor.handleExport')
    ElMessage.error('导出失败：' + msg)
  }
}

async function handleImport() {
  const { invoke } = await import('@tauri-apps/api/core')
  try {
    const data: any = await invoke('rubric_import_json')
    if (data?.questions?.length) {
      questions.value = JSON.parse(JSON.stringify(data.questions))
      activeIndex.value = 0
      ElMessage.success(`已导入 ${data.questions.length} 题`)
    }
  } catch (err) {
    const msg = String((err as any)?.message ?? err)
    logger.error(msg, 'RubricEditor.handleImport')
    ElMessage.error('导入失败：' + msg)
  }
}

async function saveAsTemplate() {
  if (!templateName.value || !templateCourse.value) {
    ElMessage.warning('请填写模板名称和课程')
    return
  }
  const { invoke } = await import('@tauri-apps/api/core')
  try {
    const ok = await invoke('template_save', {
      data: {
        name: templateName.value,
        course: templateCourse.value,
        questions: JSON.parse(JSON.stringify(questions.value))
      }
    })
    if (ok) {
      ElMessage.success('模板已保存')
      templateName.value = ''
      templateCourse.value = ''
      showTemplateForm.value = false
    }
  } catch (err) {
    const msg = String((err as any)?.message ?? err)
    logger.error(msg, 'RubricEditor.saveAsTemplate')
    ElMessage.error('保存模板失败：' + msg)
  }
}

function handleClose() {
  import('@tauri-apps/api/window').then(({ getCurrentWindow }) => getCurrentWindow().close())
}

async function uploadRefImage(idx: number) {
  const { invoke } = await import('@tauri-apps/api/core')
  const data: any = await invoke('dialog_open_image')
  if (data) {
    const mime = data.mime || 'image/png'
    const base64 = await addWatermark(data.base64, '参考答案')
    questions.value[idx].referenceAnswerImage = `data:image/${mime};base64,${base64}`
  }
}

function removeRefImage(idx: number) {
  delete questions.value[idx].referenceAnswerImage
}
</script>

<style scoped>
.dialog-header {
  display: flex; align-items: center; padding: 0;
}
.dialog-title {
  font-size: 15px; font-weight: 600;
}
.rubric-layout {
  display: flex; flex-direction: column; gap: 12px; flex: 1; min-height: 0;
}
.tab-list {
  display: flex; gap: 4px; flex-wrap: wrap; align-items: center; flex-shrink: 0;
}
.tab-item {
  display: flex; align-items: center; gap: 4px; padding: 4px 10px; border-radius: 4px;
  font-size: 13px; cursor: pointer; background: #f5f7fa; border: 1px solid #e4e7ed;
}
.tab-item.active { background: #409EFF; color: #fff; border-color: #409EFF; }
.tab-content {
  border: 1px solid #e4e7ed; border-radius: 6px; padding: 16px;
  flex: 1; overflow-y: auto; min-height: 0;
}
.footer-left { display: flex; gap: 8px; }
.footer-right { display: flex; gap: 8px; }
.ref-image-section { margin-top: 8px; display: flex; align-items: center; gap: 8px; }
.ref-image-preview { display: flex; align-items: center; gap: 8px; }
.ref-image-preview img { max-height: 120px; max-width: 200px; border-radius: 4px; border: 1px solid #e4e7ed; }
.template-form { padding: 8px 16px; border-top: 1px solid #eee; background: #fafafa; }

:deep(.el-dialog__body) {
  display: flex; flex-direction: column; flex: 1; min-height: 0; padding: 20px;
}
:deep(.el-dialog__footer) {
  display: flex; justify-content: space-between; align-items: center; flex-shrink: 0;
}
</style>
