import { beforeEach, describe, expect, it, vi } from 'vitest'

const capabilities = { features: ['daemon'], job_types: ['organize'], resources: [] }

describe('status capability cache', () => {
  beforeEach(() => {
    vi.resetModules()
    vi.stubGlobal('fetch', vi.fn(async () => new Response(JSON.stringify(capabilities), {
      status: 200,
      headers: { 'content-type': 'application/json' },
    })))
  })

  it('shares in-flight requests and reuses the loaded result', async () => {
    const { loadCapabilities } = await import('./status')

    const [first, second] = await Promise.all([loadCapabilities(), loadCapabilities()])
    const third = await loadCapabilities()

    expect(first).toEqual(capabilities)
    expect(second).toEqual(capabilities)
    expect(third).toEqual(capabilities)
    expect(fetch).toHaveBeenCalledTimes(1)
  })
})
