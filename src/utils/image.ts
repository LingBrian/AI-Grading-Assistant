export function addWatermark(base64: string, text: string): Promise<string> {
  return new Promise((resolve, reject) => {
    const img = new Image()
    img.onload = () => {
      let w = img.width; let h = img.height; const MAX = 2048; let scale = 1
      if (w > MAX || h > MAX) { scale = Math.min(MAX / w, MAX / h, 1); w = Math.round(w * scale); h = Math.round(h * scale) }
      const canvas = document.createElement('canvas'); canvas.width = w; canvas.height = h
      const ctx = canvas.getContext('2d')
      if (!ctx) return reject(new Error('Canvas 2D context not available'))
      ctx.drawImage(img, 0, 0, w, h)
      ctx.font = `bold ${Math.max(28, Math.round(w / 20))}px sans-serif`
      ctx.fillStyle = 'rgba(255, 0, 0, 0.85)'; ctx.textBaseline = 'top'
      const padding = Math.max(8, Math.round(w / 60))
      ctx.fillText(text, padding, padding)
      resolve(canvas.toDataURL('image/png').replace(/^data:image\/png;base64,/, ''))
    }
    img.onerror = () => reject(new Error('Failed to load image for watermark'))
    img.src = `data:image/png;base64,${base64}`
  })
}
