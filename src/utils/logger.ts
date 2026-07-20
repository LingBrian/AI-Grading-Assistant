export type LogLevel = 'DEBUG' | 'INFO' | 'WARN' | 'ERROR'

const LOG_ORDER: Record<LogLevel, number> = { DEBUG: 0, INFO: 1, WARN: 2, ERROR: 3 }

const ENABLED_LEVEL: LogLevel = (import.meta.env.VITE_LOG_LEVEL as LogLevel) || 'DEBUG'

function shouldLog(level: LogLevel): boolean {
  return LOG_ORDER[level] >= LOG_ORDER[ENABLED_LEVEL]
}

function now(): string {
  return new Date().toISOString()
}

async function persist(level: LogLevel, message: string, source: string): Promise<void> {
  if (level === 'DEBUG' || level === 'INFO') return
  try {
    const { invoke } = await import('@tauri-apps/api/core')
    await invoke('log_write', { level, message, source }).catch(() => {})
  } catch {}
}

const logger = {
  debug(message: string, source = 'app'): void {
    if (!shouldLog('DEBUG')) return
    console.debug(`[${now()}] [DEBUG] [${source}] ${message}`)
  },

  info(message: string, source = 'app'): void {
    if (!shouldLog('INFO')) return
    console.info(`[${now()}] [INFO] [${source}] ${message}`)
  },

  warn(message: string, source = 'app'): void {
    if (!shouldLog('WARN')) return
    console.warn(`[${now()}] [WARN] [${source}] ${message}`)
    persist('WARN', message, source)
  },

  error(message: string, source = 'app', error?: unknown): void {
    if (!shouldLog('ERROR')) return
    const detail = error ? ` | ${String((error as any)?.message ?? error)}` : ''
    const full = `${message}${detail}`
    console.error(`[${now()}] [ERROR] [${source}] ${full}`)
    persist('ERROR', full, source)
  },
}

export default logger
