import { afterEach, describe, expect, it, vi } from 'vitest'
import { api } from './api'

afterEach(() => vi.restoreAllMocks())

describe('RSS and CloudDrive API contracts', () => {
  it('posts typed RSS subscription fields with the default organization interval without browser persistence', async () => {
    const fetchMock = vi.fn().mockResolvedValue(new Response(JSON.stringify({ id: 1 }), { status: 201 }))
    vi.stubGlobal('fetch', fetchMock)
    await api.createSubscription({ url: 'https://example.test/rss', target_folder: '/anime', interval_secs: 300, connection_id: 4 })
    expect(fetchMock.mock.calls[0][0]).toBe('/api/v1/rss/subscriptions')
    expect(JSON.parse(fetchMock.mock.calls[0][1].body)).toMatchObject({ url: 'https://example.test/rss', connection_id: 4, organize_interval_secs: 300, remote_mlip: false })
    expect(Object.keys(localStorage)).toEqual([])
  })

  it('posts a separate organization interval for automatic CloudDrive organization', async () => {
    const fetchMock = vi.fn().mockResolvedValue(new Response(JSON.stringify({ id: 1 }), { status: 201 }))
    vi.stubGlobal('fetch', fetchMock)
    await api.createSubscription({ url: 'https://example.test/rss', target_folder: '/incoming', interval_secs: 900, connection_id: 4, auto_organize: true, organize_target_folder: '/anime', organize_interval_secs: 600, organize_season_mode: true, remove_empty_dirs: false, remote_mlip: true })
    expect(JSON.parse(fetchMock.mock.calls[0][1].body)).toMatchObject({ interval_secs: 900, auto_organize: true, organize_target_folder: '/anime', organize_interval_secs: 600, remote_mlip: true })
  })

  it('queues a typed manual remote organization job', async () => {
    const fetchMock = vi.fn().mockResolvedValue(new Response(JSON.stringify({ job: { id: 9 }, duplicate: false }), { status: 202 }))
    vi.stubGlobal('fetch', fetchMock)
    await api.runOrganization(7)
    expect(fetchMock.mock.calls[0][0]).toBe('/api/v1/jobs')
    expect(JSON.parse(fetchMock.mock.calls[0][1].body)).toEqual({
      origin: 'manual',
      job: { type: 'remote_rss_organize', args: { subscription_id: 7 } },
    })
  })

  it('uses the bounded folder endpoint with a breadcrumb path', async () => {
    const fetchMock = vi.fn().mockResolvedValue(new Response(JSON.stringify({ entries: [] }), { status: 200 }))
    vi.stubGlobal('fetch', fetchMock)
    await api.listFolder(4, '/Anime/Season 1')
    expect(fetchMock.mock.calls[0][0]).toBe('/api/v1/cloud/connections/4/list-folder')
    expect(JSON.parse(fetchMock.mock.calls[0][1].body)).toEqual({ path: '/Anime/Season 1' })
  })
})
