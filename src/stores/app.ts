import { defineStore } from 'pinia'
import { ref } from 'vue'

export const useAppStore = defineStore('app', () => {
  const isToolbarVisible = ref(true)
  const isResultDrawerOpen = ref(false)
  const isSettingDialogOpen = ref(false)
  const isHistoryDialogOpen = ref(false)
  const isTemplateDialogOpen = ref(false)
  const isRubricEditorOpen = ref(false)
  const isGrading = ref(false)

  function toggleToolbar() { isToolbarVisible.value = !isToolbarVisible.value }
  function openResultDrawer() { isResultDrawerOpen.value = true }
  function closeResultDrawer() { isResultDrawerOpen.value = false }
  function openSettingDialog() { isSettingDialogOpen.value = true }
  function closeSettingDialog() { isSettingDialogOpen.value = false }
  function openHistoryDialog() { isHistoryDialogOpen.value = true }
  function closeHistoryDialog() { isHistoryDialogOpen.value = false }
  function openTemplateDialog() { isTemplateDialogOpen.value = true }
  function closeTemplateDialog() { isTemplateDialogOpen.value = false }
  function openRubricEditor() { isRubricEditorOpen.value = true }
  function closeRubricEditor() { isRubricEditorOpen.value = false }

  return {
    isToolbarVisible, isResultDrawerOpen, isSettingDialogOpen, isHistoryDialogOpen,
    isTemplateDialogOpen, isRubricEditorOpen, isGrading,
    toggleToolbar, openResultDrawer, closeResultDrawer, openSettingDialog, closeSettingDialog,
    openHistoryDialog, closeHistoryDialog, openTemplateDialog, closeTemplateDialog,
    openRubricEditor, closeRubricEditor
  }
})
