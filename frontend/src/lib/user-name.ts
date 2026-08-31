import { USER_NAMES } from '@/api/mock/data'
import { getApiMode } from '@/api'

// In-memory cache populated from backend responses
const nameCache = new Map<string, string>()

export function cacheUserName(userId: string, name: string) {
  nameCache.set(userId, name)
}

/** Best-effort display name for a user id. Falls back to a short id
 *  fragment for ids the mock seed data doesn't know about (e.g. ones
 *  created via the "real" backend). */
export function userName(userId: string | null | undefined): string {
  if (!userId) return 'Unknown'
  // Check runtime cache first (populated by auth context)
  const cached = nameCache.get(userId)
  if (cached) return cached
  // In mock mode, check the hard-coded map
  if (getApiMode() === 'mock') {
    return USER_NAMES[userId] ?? `User ${userId.slice(0, 8)}`
  }
  return `User ${userId.slice(0, 8)}`
}
