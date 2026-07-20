<template>
  <div class="result-drawer" v-show="visible" @click.self="closeDrawer">
    <div class="drawer-panel">
      <div class="drawer-header" data-tauri-drag-region>
        <span class="drawer-title">AI 评分结果</span>
        <el-button text size="small" @click="closeDrawer">✕</el-button>
      </div>

      <div class="error-message" v-if="errorMsg" style="padding: 12px 16px;">{{ errorMsg }}</div>
      <div class="drawer-body" v-if="gradingStore.currentResult">
        <div class="question-nav" v-if="questions.length > 1">
          <el-button size="small" @click="prevQuestion">上一题</el-button>
          <span class="question-indicator">{{ currentIdx + 1 }} / {{ questions.length }}</span>
          <el-button size="small" @click="nextQuestion">{{ currentIdx === questions.length - 1 ? '回到首题' : '下一题' }}</el-button>
        </div>

        <div class="question-title" v-if="currentQuestion.questionTitle">
          {{ currentQuestion.questionTitle }}
        </div>
        <div class="question-maxscore" v-if="currentQuestion.maxScore">
          满分：{{ currentQuestion.maxScore }}
        </div>

        <div class="result-section" v-if="currentQuestion.rubric">
          <div class="section-title">当前评分标准</div>
          <div class="rubric-text">{{ currentQuestion.rubric }}</div>
        </div>

        <div class="result-section" v-if="gradingStore.currentImage">
          <div class="section-title">学生答案图片</div>
          <img class="student-image" :src="'data:image/png;base64,' + gradingStore.currentImage" />
        </div>

        <div class="result-section" v-if="currentQuestion.referenceAnswer || currentQuestion.referenceAnswerImage">
          <div class="section-title">参考答案</div>
          <div class="reference-text" v-if="currentQuestion.referenceAnswer">{{ currentQuestion.referenceAnswer }}</div>
          <img v-if="currentQuestion.referenceAnswerImage" class="ref-image" :src="currentQuestion.referenceAnswerImage" />
        </div>

        <div class="result-score">
          <div class="score-value">{{ gradingStore.currentResult.score }}</div>
          <div class="score-label">建议分数</div>
        </div>

        <div class="result-confidence">
          <div class="confidence-bar">
            <div class="confidence-fill" :style="{ width: (gradingStore.currentResult.confidence * 100) + '%' }"></div>
          </div>
          <div class="confidence-text">可信度：{{ Math.round(gradingStore.currentResult.confidence * 100) }}%</div>
        </div>

        <div class="result-section" v-if="gradingStore.currentResult.deductions.length">
          <div class="section-title">扣分项</div>
          <div class="deduction-item" v-for="(item, idx) in gradingStore.currentResult.deductions" :key="idx">
            {{ idx + 1 }}. {{ item }}
          </div>
        </div>

        <div class="result-section">
          <div class="section-title">评语</div>
          <div class="comment-text">{{ gradingStore.currentResult.comment }}</div>
        </div>

        <div class="result-section">
          <div class="section-title">教师评分（可修改）</div>
          <div class="teacher-score-input">
            <el-input-number v-model="teacherScore" :min="0" :max="currentQuestion.maxScore" size="small" />
          </div>
        </div>

        <div class="result-section">
          <div class="section-title">教师评语</div>
          <el-input v-model="teacherComment" type="textarea" :rows="2" size="small" placeholder="输入评语（可选）" />
        </div>
      </div>

      <div class="drawer-footer">
        <el-button size="small" @click="reGrade" :loading="isGrading">重新评分</el-button>
        <el-button type="primary" size="small" @click="confirmScore">确认评分</el-button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ElMessage } from 'element-plus'
import { ref, watch, computed } from 'vue'
import { useAppStore } from '@/stores/app'
import { useGradingStore } from '@/stores/grading'
import { useSettingsStore } from '@/stores/settings'
import type { QuestionItem } from '@/types'
import logger from '@/utils/logger'

const appStore = useAppStore()
const gradingStore = useGradingStore()
const settingsStore = useSettingsStore()

const visible = ref(false)
const isGrading = ref(false)
const errorMsg = ref('')
const teacherScore = ref(0)
const teacherComment = ref('')
const questions = ref<QuestionItem[]>([])
const currentIdx = ref(0)

const currentQuestion = computed(() => questions.value[currentIdx.value] || { questionTitle: '', maxScore: 100, referenceAnswer: '', rubric: '' })

watch(() => appStore.isResultDrawerOpen, (val) => {
  visible.value = val
})

watch(() => gradingStore.currentResult, (val) => {
  if (val) {
    teacherScore.value = val.score
    teacherComment.value = ''
  }
})

watch(visible, (val) => {
  if (val) {
    if (gradingStore.questions.length) {
      questions.value = JSON.parse(JSON.stringify(gradingStore.questions))
    }
    currentIdx.value = gradingStore.currentQuestionIndex
  }
})

function closeDrawer() {
  import('@tauri-apps/api/window').then(({ getCurrentWindow }) => getCurrentWindow().close())
}

async function gradeAt(idx: number) {
  if (!gradingStore.currentImage) return
  isGrading.value = true
  errorMsg.value = ''
  try {
    const q = questions.value[idx]
    const { invoke } = await import('@tauri-apps/api/core')
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
    gradingStore.setResult(result)
    currentIdx.value = idx
    gradingStore.setCurrentQuestionIndex(idx)
    const { invoke: inv } = await import('@tauri-apps/api/core')
    await inv('grading_store_data', {
      data: { result, image: gradingStore.currentImage, questions: questions.value, currentQuestionIndex: idx }
    })
    ElMessage.success(`第 ${idx + 1} 题评分完成`)
  } catch (err) {
    let msg = String((err as any)?.message ?? err)
    msg = msg.replace(/Error invoking '\w+':\s*/i, '')
    if (/does not support image|not support.*image|image.*input|Cannot read|image\.png/i.test(msg)) {
      msg = '当前模型不支持图片输入，请在设置中更换为支持视觉的模型'
    } else if (/does not support|model.*capable|image.*not/i.test(msg)) {
      msg = '当前模型不支持图片输入，请在设置中更换为支持视觉的模型'
    }
    logger.error(msg, 'gradeAt')
    errorMsg.value = msg
  } finally {
    isGrading.value = false
  }
}

async function nextQuestion() {
  const next = (currentIdx.value + 1) % questions.value.length
  await saveAndAdvance(next)
}

async function prevQuestion() {
  const prev = (currentIdx.value - 1 + questions.value.length) % questions.value.length
  await saveAndAdvance(prev)
}

async function saveAndAdvance(targetIdx: number) {
  const result = gradingStore.currentResult
  if (!result) return
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    const raw = {
      questionId: 0,
      imagePath: '',
      aiScore: result.score,
      teacherScore: teacherScore.value,
      confidence: result.confidence,
      studentAnswer: '',
      questionTitle: currentQuestion.value.questionTitle,
      maxScore: currentQuestion.value.maxScore,
      referenceAnswer: currentQuestion.value.referenceAnswer,
      rubric: currentQuestion.value.rubric,
      deductions: result.deductions,
      comment: result.comment,
      resultJson: JSON.stringify(result),
      imageData: gradingStore.currentImage || '',
      referenceAnswerImage: currentQuestion.value.referenceAnswerImage || ''
    }
    await invoke('grading_submit', { data: JSON.parse(JSON.stringify(raw)) })
    await invoke('rubric_set_next_index', { idx: targetIdx })
  } catch (err) {
    const msg = String((err as any)?.message ?? err)
    logger.error(msg, 'saveAndAdvance')
    ElMessage.error('保存失败：' + msg)
  }
  closeDrawer()
}

async function reGrade() {
  await gradeAt(currentIdx.value)
}

async function confirmScore() {
  errorMsg.value = ''
  const result = gradingStore.currentResult
  if (!result) {
    errorMsg.value = '暂无评分结果可保存（currentResult 为空）'
    return
  }
  const { invoke } = await import('@tauri-apps/api/core')
  try {
    const raw = {
      questionId: 0,
      imagePath: '',
      aiScore: result.score,
      teacherScore: teacherScore.value,
      confidence: result.confidence,
      studentAnswer: '',
      questionTitle: currentQuestion.value.questionTitle,
      maxScore: currentQuestion.value.maxScore,
      referenceAnswer: currentQuestion.value.referenceAnswer,
      rubric: currentQuestion.value.rubric,
      deductions: result.deductions,
      comment: result.comment,
      resultJson: JSON.stringify(result),
      imageData: gradingStore.currentImage || '',
      referenceAnswerImage: currentQuestion.value.referenceAnswerImage || ''
    }
    await invoke('grading_submit', { data: JSON.parse(JSON.stringify(raw)) })
    gradingStore.reset()
    closeDrawer()
  } catch (err) {
    const msg = String((err as any)?.message ?? err)
    logger.error(msg, 'confirmScore')
    errorMsg.value = '保存失败：' + msg
  }
}
</script>

<style scoped>
.result-drawer { position: fixed; inset: 0; z-index: 99998; background: #fff; display: flex; }
.drawer-panel { width: 100%; height: 100%; background: #fff; display: flex; flex-direction: column; }
.drawer-header { display: flex; justify-content: space-between; align-items: center; padding: 16px; border-bottom: 1px solid #eee; }
.drawer-title { font-size: 16px; font-weight: 600; }
.drawer-body { flex: 1; overflow-y: auto; padding: 16px; }
.question-nav { display: flex; align-items: center; justify-content: space-between; margin-bottom: 12px; }
.question-indicator { font-size: 14px; font-weight: 600; color: #409EFF; }
.question-title { font-size: 14px; font-weight: 600; color: #333; margin-bottom: 4px; }
.question-maxscore { font-size: 12px; color: #999; margin-bottom: 12px; }
.result-score { text-align: center; padding: 16px 0; }
.score-value { font-size: 48px; font-weight: 700; color: #409EFF; line-height: 1; }
.score-label { font-size: 13px; color: #999; margin-top: 4px; }
.result-confidence { margin-bottom: 16px; }
.confidence-bar { height: 6px; background: #eee; border-radius: 3px; overflow: hidden; }
.confidence-fill { height: 100%; background: linear-gradient(90deg, #67C23A, #409EFF); border-radius: 3px; transition: width 0.3s; }
.confidence-text { font-size: 12px; color: #999; margin-top: 4px; text-align: right; }
.result-section { margin-bottom: 16px; }
.section-title { font-size: 13px; font-weight: 600; color: #333; margin-bottom: 8px; }
.deduction-item { font-size: 13px; color: #F56C6C; padding: 4px 0; line-height: 1.5; }
.comment-text { font-size: 13px; color: #666; line-height: 1.6; background: #f5f7fa; padding: 10px; border-radius: 6px; }
.rubric-text { font-size: 13px; color: #333; line-height: 1.6; background: #fff7e6; padding: 10px; border-radius: 6px; white-space: pre-wrap; }
.reference-text { font-size: 13px; color: #666; line-height: 1.6; background: #f0f9eb; padding: 10px; border-radius: 6px; white-space: pre-wrap; }
.ref-image, .student-image { max-width: 100%; max-height: 300px; border-radius: 6px; margin-top: 8px; border: 1px solid #e4e7ed; }
.error-message { background: #fef0f0; color: #f56c6c; border: 1px solid #fde2e2; border-radius: 6px; padding: 10px; font-size: 13px; line-height: 1.5; margin-bottom: 12px; }
.teacher-score-input { margin-top: 4px; }
.drawer-footer { padding: 12px 16px; border-top: 1px solid #eee; display: flex; gap: 8px; justify-content: flex-end; }
</style>
