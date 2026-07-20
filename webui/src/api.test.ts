import { afterEach, describe, expect, it, vi } from 'vitest'
import { ApiError, api } from './api'
import { jobQueryState } from './filters'

afterEach(() => vi.restoreAllMocks())

describe('API errors and job filters', () => {
  it('turns an API error envelope into a useful field-ready error', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue(new Response(JSON.stringify({ error: { code: 'invalid_request', message: 'target is required' } }), { status: 422 })))
    await expect(api.health()).rejects.toEqual(new ApiError('invalid_request', 'target is required', 422))
  })

  it('rejects a successful non-JSON API response', async () => {
    vi.stubGlobal('fetch', vi.fn().mockResolvedValue(new Response('<html></html>', { status: 200 })))
    await expect(api.health()).rejects.toEqual(new ApiError('invalid_response', 'API returned an invalid response', 200))
  })

  it('accepts only known URL job states', () => {
    expect(jobQueryState('failed')).toBe('failed')
    expect(jobQueryState('unknown')).toBeUndefined()
    expect(jobQueryState('')).toBeUndefined()
  })

  it('submits typed alias maintenance jobs with confirmation', async () => {
    const fetchMock = vi.fn().mockImplementation(async () => new Response(JSON.stringify({ job: { id: 8 }, duplicate: false }), { status: 202 }))
    vi.stubGlobal('fetch', fetchMock)
    await api.enqueueMergeAliases({ input: 'aliases.json', target: 'selected.db' }, true)
    await api.enqueueApplyMatches({ input: 'matches.json', target: 'selected.db' }, false)
    const mergeBody = JSON.parse(fetchMock.mock.calls[0][1].body as string)
    const applyBody = JSON.parse(fetchMock.mock.calls[1][1].body as string)
    expect(mergeBody.confirmed).toBe(true)
    expect(mergeBody.job).toEqual({ type: 'merge_aliases', args: { input: 'aliases.json', target: 'selected.db' } })
    expect(applyBody.confirmed).toBe(false)
    expect(applyBody.job.type).toBe('apply_matches')
  })

  it('submits typed scraper jobs without reading command output', async () => {
    const fetchMock = vi.fn().mockImplementation(async () => new Response(JSON.stringify({ job: { id: 7 }, duplicate: false }), { status: 202 }))
    vi.stubGlobal('fetch', fetchMock)
    await api.enqueueScrape({ days: 7, format: 'json', tmdb_api_key: null })
    await api.enqueueMatchAliases({ input: 'scraped.json', format: 'github' })
    const scrapeBody = JSON.parse(fetchMock.mock.calls[0][1].body as string)
    const matchBody = JSON.parse(fetchMock.mock.calls[1][1].body as string)
    expect(scrapeBody.job.type).toBe('scrape')
    expect(matchBody.job).toEqual({ type: 'match_aliases', args: { input: 'scraped.json', format: 'github' } })
  })

  it('submits a CloudDrive offline job without credentials', async () => {
    const fetchMock = vi.fn().mockResolvedValue(new Response(JSON.stringify({ job: { id: 10 }, duplicate: false }), { status: 202 }))
    vi.stubGlobal('fetch', fetchMock)
    await api.enqueueCloudAddOffline({ connection_id: 3, url: 'magnet:?xt=test', target: '/anime' })
    const body = JSON.parse(fetchMock.mock.calls[0][1].body as string)
    expect(body.job).toEqual({ type: 'cloud_add_offline', args: { connection_id: 3, url: 'magnet:?xt=test', target: '/anime' } })
    expect(JSON.stringify(body)).not.toContain('token')
  })

  it('submits typed torrent source parameters', async () => {
    const fetchMock = vi.fn().mockResolvedValue(new Response(JSON.stringify({ job: { id: 9 }, duplicate: false }), { status: 202 }))
    vi.stubGlobal('fetch', fetchMock)
    await api.enqueueTorrentScrape({ source: 'nyaa', query: 'anime', pages: 2000, output: null, headed: false })
    const body = JSON.parse(fetchMock.mock.calls[0][1].body as string)
    expect(body.job).toEqual({ type: 'torrent_scrape', args: { source: 'nyaa', query: 'anime', pages: 2000, output: null, headed: false } })
  })
})
