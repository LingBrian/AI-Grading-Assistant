export interface QuestionItem {
  questionTitle: string
  maxScore: number
  referenceAnswer: string
  rubric: string
  referenceAnswerImage?: string
}

export interface GradeResult {
  score: number
  confidence: number
  deductions: string[]
  comment: string
  commit?: string
}

export interface ApiConfig {
  apiKey: string
  baseUrl: string
  model: string
}

export interface AppSettings {
  apiKey: string
  baseUrl: string
  model: string
  shortcut: string
  theme: string
  fontSize: number
  proxy: string
  timeout: number
  screenshotSavePath: string
}

export interface HistoryRecord {
  id: number
  question_id: number
  image_path: string
  student_answer: string
  ai_score: number
  teacher_score: number
  confidence: number
  created_at: string
  question_title?: string
  max_score?: number
  reference_answer?: string
  rubric?: string
  deductions?: string
  comment?: string
  result_json?: string
  image_data?: string
  reference_answer_image?: string
}

export interface TemplateItem {
  id: number
  name: string
  course: string
  questions: QuestionItem[]
  created_at: string
}
