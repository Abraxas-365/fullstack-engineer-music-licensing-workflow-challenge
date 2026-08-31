import { USER_NAMES } from '@/api/mock/data'

/** Best-effort display name for a user id. Falls back to a short id
 *  fragment for ids the mock seed data doesn't know about (e.g. ones
 *  created via the "real" backend). */
export function userName(userId: string | null | undefined): string {
  if (!userId) return 'Unknown'
  return USER_NAMES[userId] ?? `User ${userId.slice(0, 8)}`
}
