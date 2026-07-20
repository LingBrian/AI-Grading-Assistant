<template>
  <el-dialog v-model="visible" fullscreen :modal="false" @close="handleClose">
    <template #header>
      <div class="dialog-header" data-tauri-drag-region>
        <span class="dialog-title">历史记录</span>
      </div>
    </template>
    <el-table :data="records" style="width: 100%" size="small" v-loading="loading">
      <el-table-column prop="id" label="ID" width="60" />
      <el-table-column prop="ai_score" label="AI分" width="70" />
      <el-table-column prop="teacher_score" label="教师分" width="70" />
      <el-table-column prop="confidence" label="可信度" width="70">
        <template #default="{ row }">
          {{ Math.round(row.confidence * 100) }}%
        </template>
      </el-table-column>
      <el-table-column prop="question_title" label="题目" min-width="120" show-overflow-tooltip />
      <el-table-column prop="created_at" label="时间" width="150" />
      <el-table-column label="操作" width="140" fixed="right">
        <template #default="{ row }">
          <el-button text size="small" type="primary" @click="viewResult(row)">查看</el-button>
          <el-button text size="small" type="danger" @click="deleteRecord(row.id)">删除</el-button>
        </template>
      </el-table-column>
    </el-table>

    <template #footer>
      <el-button size="small" @click="handleClose">关闭</el-button>
      <el-button size="small" @click="refresh">刷新</el-button>
    </template>
  </el-dialog>

  <div v-if="detailVisible" class="history-result-overlay" @click.self="detailVisible = false">
    <div class="history-result-panel">
      <div class="panel-header">
        <span class="panel-title">评分详情</span>
        <el-button text size="small" @click="detailVisible = false">✕</el-button>
      </div>
      <div class="panel-body" v-if="detail">
        <div class="question-title" v-if="detail.question_title">
          {{ detail.question_title }}
        </div>
        <div class="question-maxscore" v-if="detail.max_score">
          满分：{{ detail.max_score }}
        </div>

        <div class="result-section" v-if="detail.rubric">
          <div class="section-title">当前评分标准</div>
          <div class="rubric-text">{{ detail.rubric }}</div>
        </div>

        <div class="result-section" v-if="detail.image_data">
          <div class="section-title">学生答案图片</div>
          <img class="result-image" :src="'data:image/png;base64,' + detail.image_data" />
        </div>

        <div class="result-section" v-if="detail.reference_answer || detail.reference_answer_image">
          <div class="section-title">参考答案</div>
          <div class="reference-text" v-if="detail.reference_answer">{{ detail.reference_answer }}</div>
          <img v-if="detail.reference_answer_image" class="result-image" :src="detail.reference_answer_image" />
        </div>

        <div class="result-score">
          <div class="score-value">{{ detail.ai_score }}</div>
          <div class="score-label">建议分数</div>
        </div>

        <div class="result-confidence">
          <div class="confidence-bar">
            <div class="confidence-fill" :style="{ width: Math.round(detail.confidence * 100) + '%' }"></div>
          </div>
          <div class="confidence-text">可信度：{{ Math.round(detail.confidence * 100) }}%</div>
        </div>

        <div class="result-section" v-if="deductionsList.length">
          <div class="section-title">扣分项</div>
          <div class="deduction-item" v-for="(item, idx) in deductionsList" :key="idx">
            {{ Number(idx) + 1 }}. {{ item }}
          </div>
        </div>

        <div class="result-section" v-if="detail.comment">
          <div class="section-title">评语</div>
          <div class="comment-text">{{ detail.comment }}</div>
        </div>

        <div class="result-section">
          <div class="section-title">教师评分</div>
          <div class="teacher-score-display">{{ detail.teacher_score }} / {{ detail.max_score || 100 }}</div>
        </div>
      </div>
      <div class="panel-footer">
        <el-button size="small" @click="detailVisible = false">关闭</el-button>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, watch } from 'vue'
import { ElMessage, ElMessageBox } from 'element-plus'
import { useAppStore } from '@/stores/app'
import type { HistoryRecord } from '@/types'
import logger from '@/utils/logger'

const appStore = useAppStore()
const visible = ref(false)
const loading = ref(false)
const records = ref<HistoryRecord[]>([])

const detailVisible = ref(false)
const detail = ref<HistoryRecord | null>(null)
const deductionsList = computed(() => {
  if (!detail.value?.deductions) return []
  try { return JSON.parse(detail.value.deductions) } catch { return [] }
})

watch(() => appStore.isHistoryDialogOpen, (val) => {
  visible.value = val
  if (val) refresh()
})

async function refresh() {
  loading.value = true
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    records.value = await invoke('history_list')
  } catch (e) {
    logger.error('refresh failed', 'HistoryDialog', e)
    records.value = []
  } finally {
    loading.value = false
  }
}

async function viewResult(row: HistoryRecord) {
  const { invoke } = await import('@tauri-apps/api/core')
  try {
    const full: any = await invoke('history_get_by_id', { id: row.id })
    if (full) {
      detail.value = full
      detailVisible.value = true
    } else {
      ElMessage.warning('未找到记录详情')
    }
  } catch (err) {
    const msg = String((err as any)?.message ?? err)
    logger.error(msg, 'HistoryDialog.loadDetail')
    ElMessage.error('加载详情失败：' + msg)
  }
}

async function deleteRecord(id: number) {
  try {
    await ElMessageBox.confirm('确定删除该记录？', '提示')
    const { invoke } = await import('@tauri-apps/api/core')
    await invoke('history_delete', { id })
    ElMessage.success('已删除')
    refresh()
  } catch { /* cancelled */ }
}

function handleClose() {
  import('@tauri-apps/api/window').then(({ getCurrentWindow }) => getCurrentWindow().close())
}
</script>

<style scoped>
.history-result-overlay { position: fixed; inset: 0; z-index: 99998; background: rgba(0, 0, 0, 0.3); display: flex; justify-content: flex-end; }
.history-result-panel { width: 480px; height: 100%; background: #fff; display: flex; flex-direction: column; box-shadow: -4px 0 24px rgba(0, 0, 0, 0.15); }
.panel-header { display: flex; justify-content: space-between; align-items: center; padding: 14px 16px; border-bottom: 1px solid #eee; font-size: 15px; font-weight: 600; }
.panel-body { flex: 1; overflow-y: auto; padding: 16px; display: flex; flex-direction: column; gap: 14px; }
.panel-footer { padding: 12px 16px; border-top: 1px solid #eee; display: flex; justify-content: flex-end; }
.question-title { font-size: 14px; font-weight: 600; color: #333; line-height: 1.5; }
.question-maxscore { font-size: 12px; color: #909399; }
.result-section { display: flex; flex-direction: column; gap: 6px; }
.section-title { font-size: 12px; font-weight: 600; color: #909399; }
.rubric-text { font-size: 13px; color: #333; line-height: 1.6; background: #fff7e6; padding: 10px; border-radius: 6px; white-space: pre-wrap; }
.reference-text { font-size: 13px; color: #666; line-height: 1.6; background: #f0f9eb; padding: 10px; border-radius: 6px; white-space: pre-wrap; }
.result-image { max-width: 100%; max-height: 300px; border-radius: 6px; border: 1px solid #e4e7ed; }
.result-score { text-align: center; padding: 12px 0; }
.score-value { font-size: 42px; font-weight: 700; color: #409EFF; line-height: 1; }
.score-label { font-size: 12px; color: #909399; margin-top: 4px; }
.result-confidence { display: flex; flex-direction: column; gap: 4px; }
.confidence-bar { height: 6px; background: #e4e7ed; border-radius: 3px; overflow: hidden; }
.confidence-fill { height: 100%; background: #67c23a; border-radius: 3px; transition: width 0.3s; }
.confidence-text { font-size: 11px; color: #909399; text-align: right; }
.deduction-item { font-size: 13px; color: #f56c6c; line-height: 1.6; padding: 2px 0; }
.comment-text { font-size: 13px; color: #333; line-height: 1.6; background: #f5f7fa; padding: 10px; border-radius: 6px; white-space: pre-wrap; }
.teacher-score-display { font-size: 16px; font-weight: 600; color: #409EFF; }
</style>
