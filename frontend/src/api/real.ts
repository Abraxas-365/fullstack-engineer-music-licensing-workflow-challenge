import { del, get, getAccessToken, getBaseUrl, post, put, setAccessToken } from './http'
import type {
  Api,
  AuthApi,
  LabelsApi,
  LicensesApi,
  MoviesApi,
  ScenesApi,
  SongsApi,
  TracksApi,
} from './types'
import type { LicenseEvent } from '@/types'

const movies: MoviesApi = {
  create: body => post('/movies', body),
  find: query => get('/movies', query),
  get: id => get(`/movies/${id}`),
  update: (id, body) => put(`/movies/${id}`, body),
  delete: id => del(`/movies/${id}`),
  myMovies: () => get('/movies/me'),
  addMember: (id, body) => post(`/movies/${id}/members`, body),
  removeMember: (id, userId) => del(`/movies/${id}/members/${userId}`),
  listMembers: id => get(`/movies/${id}/members`),
  listScenes: id => get(`/movies/${id}/scenes`),
}

const scenes: ScenesApi = {
  create: body => post('/scenes', body),
  get: id => get(`/scenes/${id}`),
  update: (id, body) => put(`/scenes/${id}`, body),
  delete: id => del(`/scenes/${id}`),
  listTracks: id => get(`/scenes/${id}/tracks`),
}

const songs: SongsApi = {
  create: body => post('/songs', body),
  find: query => get('/songs', query),
  get: id => get(`/songs/${id}`),
  update: (id, body) => put(`/songs/${id}`, body),
  delete: id => del(`/songs/${id}`),
  listByArtist: artistId => get(`/artists/${artistId}/songs`),
  listTracks: id => get(`/songs/${id}/tracks`),
}

const tracks: TracksApi = {
  create: body => post('/tracks', body),
  get: id => get(`/tracks/${id}`),
  update: (id, body) => put(`/tracks/${id}`, body),
  delete: id => del(`/tracks/${id}`),
  getLicense: id => get(`/tracks/${id}/license`),
}

const labels: LabelsApi = {
  create: body => post('/labels', body),
  list: () => get('/labels'),
  get: id => get(`/labels/${id}`),
  update: (id, body) => put(`/labels/${id}`, body),
  delete: id => del(`/labels/${id}`),
  addMember: (id, body) => post(`/labels/${id}/members`, body),
  removeMember: (id, userId) => del(`/labels/${id}/members/${userId}`),
  listMembers: id => get(`/labels/${id}/members`),
  getUserLabels: userId => get(`/users/${userId}/labels`),
  listSongs: id => get(`/labels/${id}/songs`),
}

const licenses: LicensesApi = {
  create: body => post('/licenses', body),
  get: id => get(`/licenses/${id}`),
  listOffers: id => get(`/licenses/${id}/offers`),
  reviseDraft: (id, body) => post(`/licenses/${id}/revise`, body),
  submit: id => post(`/licenses/${id}/submit`),
  counterOffer: (id, body) => post(`/licenses/${id}/counter`, body),
  accept: id => post(`/licenses/${id}/accept`),
  reject: (id, reason) => post(`/licenses/${id}/reject`, { reason }),
  cancel: id => post(`/licenses/${id}/cancel`),
  delete: id => del(`/licenses/${id}`),
  subscribeEvents(onEvent) {
    // EventSource can't send an Authorization header, and the backend only
    // reads the bearer token from that header (or a cookie), so we stream
    // the SSE response manually via fetch instead.
    const controller = new AbortController()
    const token = getAccessToken()

    void (async () => {
      try {
        const url = new URL(getBaseUrl() + '/licenses/events', window.location.origin).toString()
        const res = await fetch(url, {
          headers: token ? { Authorization: `Bearer ${token}` } : undefined,
          signal: controller.signal,
        })
        if (!res.body) return
        const reader = res.body.getReader()
        const decoder = new TextDecoder()
        let buffer = ''

        while (true) {
          const { value, done } = await reader.read()
          if (done) break
          buffer += decoder.decode(value, { stream: true })

          let sepIndex: number
          while ((sepIndex = buffer.indexOf('\n\n')) !== -1) {
            const chunk = buffer.slice(0, sepIndex)
            buffer = buffer.slice(sepIndex + 2)
            const line = chunk.split('\n').find(l => l.startsWith('data: '))
            if (!line) continue
            try {
              onEvent(JSON.parse(line.slice('data: '.length)) as LicenseEvent)
            } catch {
              // ignore malformed events
            }
          }
        }
      } catch {
        // stream closed / aborted
      }
    })()

    return () => controller.abort()
  },
}

const auth: AuthApi = {
  async login(body) {
    const res = await post<Awaited<ReturnType<AuthApi['login']>>>('/auth/login', body)
    setAccessToken(res.access_token)
    return res
  },
  async refresh(refreshToken) {
    const res = await post<Awaited<ReturnType<AuthApi['refresh']>>>('/auth/refresh', { refresh_token: refreshToken })
    setAccessToken(res.access_token)
    return res
  },
  async logout(refreshToken) {
    await post('/auth/logout', { refresh_token: refreshToken })
    setAccessToken(null)
  },
  async logoutAll() {
    await post('/auth/logout-all')
    setAccessToken(null)
  },
  me: () => get('/auth/me'),
  listSessions: () => get('/auth/sessions'),
  revokeSession: id => del(`/auth/sessions/${id}`),
}

export const realApi: Api = { auth, movies, scenes, songs, tracks, labels, licenses }
