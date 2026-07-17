import { afterEach, describe, expect, it, vi } from 'vitest'
import { api } from './api'

afterEach(() => vi.restoreAllMocks())

describe('RSS and CloudDrive API contracts', () => {
  it('posts typed RSS subscription fields without browser persistence', async () => {
    const fetchMock = vi.fn().mockResolvedValue(new Response(JSON.stringify({ id: 1 }), { status: 201 }))
    vi.stubGlobal('fetch', fetchMock)
    await api.createSubscription({ url: 'https://example.test/rss', target_folder: '/anime', interval_secs: 300, connection_id: 4 })
    expect(fetchMock.mock.calls[0][0]).toBe('/api/v1/rss/subscriptions')
    expect(JSON.parse(fetchMock.mock.calls[0][1].body)).toMatchObject({ url: 'https://example.test/rss', connection_id: 4 })
    expect(Object.keys(localStorage)).toEqual([])
  })

  it('uses the bounded folder endpoint with a breadcrumb path', async () => {
    const fetchMock = vi.fn().mockResolvedValue(new Response(JSON.stringify({ entries: [] }), { status: 200 }))
    vi.stubGlobal('fetch', fetchMock)
    await api.listFolder(4, '/Anime/Season 1')
    expect(fetchMock.mock.calls[0][0]).toBe('/api/v1/cloud/connections/4/list-folder')
    expect(JSON.parse(fetchMock.mock.calls[0][1].body)).toEqual({ path: '/Anime/Season 1' })
  })
})
