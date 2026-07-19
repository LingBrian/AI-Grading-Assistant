<template>
  <div class="screenshot-overlay" @mousedown="onMouseDown" @mousemove="onMouseMove" @mouseup="onMouseUp" :class="{ 'has-selection': hasSelection }">
    <canvas ref="canvasRef" class="screenshot-canvas"></canvas>
    <div v-if="!hasSelection" class="screenshot-tip">拖动选择截图区域 · Esc取消</div>
    <div v-else class="screenshot-actions" :style="actionsStyle" @mousedown.stop @mouseup.stop>
      <button class="action-btn confirm" @click="confirmCapture">✓ 确认</button>
      <button class="action-btn cancel" @click="cancel">✕ 取消</button>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, computed } from 'vue'

const canvasRef = ref<HTMLCanvasElement | null>(null)
const state = { isSelecting: false, startX: 0, startY: 0, endX: 0, endY: 0 }
const hasSelection = ref(false)
const isCapturing = ref(false)

const actionsStyle = computed(() => {
  if (!hasSelection.value) return {}
  const x = Math.min(state.startX, state.endX)
  const y = Math.min(state.startY, state.endY)
  const w = Math.abs(state.endX - state.startX)
  const h = Math.abs(state.endY - state.startY)
  const panelW = 160; const gap = 8
  let left = x + w - panelW; let top = y + h + gap
  if (left < 4) left = 4
  if (top + 40 > window.innerHeight) top = y - 40 - gap
  return { left: left + 'px', top: top + 'px', transform: 'none', bottom: 'auto' }
})

onMounted(() => document.addEventListener('keydown', onKeyDown))
onUnmounted(() => document.removeEventListener('keydown', onKeyDown))

function onMouseDown(e: MouseEvent) {
  hasSelection.value = false; state.isSelecting = true
  state.startX = e.clientX; state.startY = e.clientY
  state.endX = e.clientX; state.endY = e.clientY
}
function onMouseMove(e: MouseEvent) {
  if (!state.isSelecting) return
  state.endX = e.clientX; state.endY = e.clientY; drawOverlay()
}
function onMouseUp() {
  if (!state.isSelecting) return; state.isSelecting = false
  const w = Math.abs(state.endX - state.startX); const h = Math.abs(state.endY - state.startY)
  if (w < 10 || h < 10) return
  hasSelection.value = true; drawOverlay()
}
async function confirmCapture() {
  const x = Math.min(state.startX, state.endX); const y = Math.min(state.startY, state.endY)
  const width = Math.abs(state.endX - state.startX); const height = Math.abs(state.endY - state.startY)
  if (width < 10 || height < 10) return; if (isCapturing.value) return; isCapturing.value = true
  const { invoke } = await import('@tauri-apps/api/core')
  await invoke('screenshot_capture', { region: { x, y, width, height } }).catch(() => {})
  const { getCurrentWindow } = await import('@tauri-apps/api/window')
  try { await getCurrentWindow().close() } catch {}
}
async function cancel() {
  const { invoke } = await import('@tauri-apps/api/core')
  await invoke('screenshot_cancel').catch(() => {})
  isCapturing.value = false
  const { getCurrentWindow } = await import('@tauri-apps/api/window')
  try { await getCurrentWindow().close() } catch {}
}
function drawOverlay() {
  const canvas = canvasRef.value; if (!canvas) return
  const ctx = canvas.getContext('2d'); if (!ctx) return
  canvas.width = window.innerWidth; canvas.height = window.innerHeight
  ctx.clearRect(0, 0, canvas.width, canvas.height)
  const x = Math.min(state.startX, state.endX); const y = Math.min(state.startY, state.endY)
  const w = Math.abs(state.endX - state.startX); const h = Math.abs(state.endY - state.startY)
  ctx.fillStyle = 'rgba(0, 0, 0, 0.45)'; ctx.fillRect(0, 0, canvas.width, canvas.height)
  ctx.clearRect(x, y, w, h)
  ctx.strokeStyle = '#409EFF'; ctx.lineWidth = 2.5; ctx.strokeRect(x, y, w, h)
  ctx.fillStyle = 'rgba(64, 158, 255, 0.08)'; ctx.fillRect(x, y, w, h)
  ctx.fillStyle = 'rgba(255, 255, 255, 0.9)'; ctx.font = '13px sans-serif'
  ctx.fillText(`${w} × ${h}`, x + 6, y - 8)
}
function onKeyDown(e: KeyboardEvent) {
  if (e.key === 'Escape') { e.preventDefault(); e.stopPropagation(); cancel() }
  if (e.key === 'Enter' && hasSelection.value) { e.preventDefault(); e.stopPropagation(); confirmCapture() }
}
</script>

<style scoped>
.screenshot-overlay { position: fixed; inset: 0; z-index: 99999; cursor: crosshair; }
.screenshot-overlay.has-selection { cursor: default; }
.screenshot-canvas { position: fixed; inset: 0; width: 100%; height: 100%; }
.screenshot-tip { position: fixed; bottom: 40px; left: 50%; transform: translateX(-50%); background: rgba(0,0,0,0.7); color: #fff; padding: 8px 16px; border-radius: 6px; font-size: 13px; pointer-events: none; }
.screenshot-actions { position: fixed; display: flex; gap: 10px; pointer-events: auto; }
.action-btn { border: none; padding: 8px 20px; border-radius: 6px; font-size: 13px; cursor: pointer; transition: all 0.15s; }
.action-btn.confirm { background: #409EFF; color: #fff; }
.action-btn.confirm:hover { background: #66b1ff; }
.action-btn.cancel { background: rgba(0,0,0,0.6); color: #fff; }
.action-btn.cancel:hover { background: rgba(0,0,0,0.8); }
</style>
