import { useEffect, useState } from 'react'
import { type ApiMode, getApiMode, setApiMode, subscribeApiMode } from '@/api'

/** Reactive binding to the global API mode (mock vs real). */
export function useApiMode(): [ApiMode, (mode: ApiMode) => void] {
  const [mode, setMode] = useState<ApiMode>(getApiMode())

  useEffect(() => subscribeApiMode(() => setMode(getApiMode())), [])

  return [mode, setApiMode]
}
