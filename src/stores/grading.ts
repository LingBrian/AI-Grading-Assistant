import { defineStore } from 'pinia'
import { ref } from 'vue'
import type { GradeResult, QuestionItem } from '@/types'

export const useGradingStore = defineStore('grading', () => {
  const currentImage = ref<string | null>(null)
  const currentResult = ref<GradeResult | null>(null)
  const questions = ref<QuestionItem[]>([])
  const currentQuestionIndex = ref(0)
  const rubric = ref('')
  const questionTitle = ref('')
  const maxScore = ref(100)
  const referenceAnswer = ref('')
  const teacherScore = ref<number | null>(null)
  const teacherComment = ref('')

  function setImage(base64: string) { currentImage.value = base64 }
  function setResult(result: GradeResult) { currentResult.value = result }
  function setRubric(text: string) { rubric.value = text }
  function setQuestionTitle(text: string) { questionTitle.value = text }
  function setMaxScore(val: number) { maxScore.value = val }
  function setReferenceAnswer(val: string) { referenceAnswer.value = val }
  function setQuestions(list: QuestionItem[]) { questions.value = list }
  function setCurrentQuestionIndex(idx: number) { currentQuestionIndex.value = idx }

  function reset() {
    currentImage.value = null
    currentResult.value = null
    teacherScore.value = null
    teacherComment.value = ''
  }

  return {
    currentImage, currentResult, questions, currentQuestionIndex, rubric,
    questionTitle, maxScore, referenceAnswer, teacherScore, teacherComment,
    setImage, setResult, setRubric, setQuestionTitle, setMaxScore, setReferenceAnswer,
    setQuestions, setCurrentQuestionIndex, reset
  }
})
