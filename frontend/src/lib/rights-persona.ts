import { useSyncExternalStore } from 'react'
import { labels, USERS } from '@/api/mock/data'
import type { LabelRole, PlatformRole } from '@/types'

export type RightsPersonaId = 'label-owner' | 'label-rep' | 'label-artist' | 'independent-artist'

export interface RightsPersona {
  id: RightsPersonaId
  kind: 'label' | 'independent'
  user: { id: string; name: string; email: string }
  platformRole: PlatformRole
  labelId: string | null
  labelName: string | null
  labelRole: LabelRole | null
  canNegotiate: boolean
  canManageMembers: boolean
  catalogScope: 'label' | 'artist'
  title: string
}

export const RIGHTS_PERSONAS: Record<RightsPersonaId, RightsPersona> = {
  'label-owner': {
    id: 'label-owner',
    kind: 'label',
    user: USERS.labelManager,
    platformRole: 'Label Manager',
    labelId: labels[0].id,
    labelName: labels[0].name,
    labelRole: 'OWNER',
    canNegotiate: true,
    canManageMembers: true,
    catalogScope: 'label',
    title: `${labels[0].name} · Owner`,
  },
  'label-rep': {
    id: 'label-rep',
    kind: 'label',
    user: USERS.labelRep,
    platformRole: 'Label Manager',
    labelId: labels[0].id,
    labelName: labels[0].name,
    labelRole: 'REP',
    canNegotiate: true,
    canManageMembers: false,
    catalogScope: 'label',
    title: `${labels[0].name} · Rep`,
  },
  'label-artist': {
    id: 'label-artist',
    kind: 'label',
    user: USERS.artist,
    platformRole: 'Artist',
    labelId: labels[0].id,
    labelName: labels[0].name,
    labelRole: 'ARTIST',
    canNegotiate: false,
    canManageMembers: false,
    catalogScope: 'artist',
    title: `${labels[0].name} · Artist`,
  },
  'independent-artist': {
    id: 'independent-artist',
    kind: 'independent',
    user: USERS.artist,
    platformRole: 'Artist',
    labelId: null,
    labelName: null,
    labelRole: null,
    canNegotiate: true,
    canManageMembers: false,
    catalogScope: 'artist',
    title: 'Independent catalog',
  },
}

const STORAGE_KEY = 'rights-persona'
let currentId: RightsPersonaId = readInitialPersona()
const listeners = new Set<() => void>()

function readInitialPersona(): RightsPersonaId {
  if (typeof window === 'undefined') return 'label-owner'
  const stored = window.localStorage.getItem(STORAGE_KEY)
  return stored && stored in RIGHTS_PERSONAS ? stored as RightsPersonaId : 'label-owner'
}

export function getRightsPersona(): RightsPersona {
  return RIGHTS_PERSONAS[currentId]
}

export function setRightsPersona(id: RightsPersonaId) {
  if (currentId === id) return
  currentId = id
  window.localStorage.setItem(STORAGE_KEY, id)
  listeners.forEach(listener => listener())
}

function subscribe(listener: () => void) {
  listeners.add(listener)
  return () => listeners.delete(listener)
}

export function useRightsPersona(): RightsPersona {
  const id = useSyncExternalStore<RightsPersonaId>(subscribe, () => currentId, () => 'label-owner')
  return RIGHTS_PERSONAS[id]
}
