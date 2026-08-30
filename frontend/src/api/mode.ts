// Global switch between the real backend and the in-memory mock backend.
// Persisted in localStorage so a reload keeps the chosen mode.

export type ApiMode = 'mock' | 'real'

const STORAGE_KEY = 'api-mode'
const DEFAULT_MODE: ApiMode = 'mock'

function readStoredMode(): ApiMode {
  const stored = localStorage.getItem(STORAGE_KEY)
  return stored === 'real' || stored === 'mock' ? stored : DEFAULT_MODE
}

let currentMode: ApiMode = readStoredMode()
const listeners = new Set<() => void>()

export function getApiMode(): ApiMode {
  return currentMode
}

export function setApiMode(mode: ApiMode) {
  if (mode === currentMode) return
  currentMode = mode
  localStorage.setItem(STORAGE_KEY, mode)
  listeners.forEach(listener => listener())
}

export function subscribeApiMode(listener: () => void): () => void {
  listeners.add(listener)
  return () => listeners.delete(listener)
}
