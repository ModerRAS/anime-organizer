import { afterEach, describe, expect, it, vi } from 'vitest'

const cleanups: Array<() => void> = []

afterEach(() => {
  cleanups.splice(0).forEach(cleanup => cleanup())
  document.body.replaceChildren()
  window.localStorage.clear()
  vi.unstubAllGlobals()
  vi.restoreAllMocks()
})

async function mountOrganizeView(jobTypes: string[]) {
  vi.resetModules()
  window.localStorage.setItem('anime-organizer.locale', 'en')

  const fetchMock = vi.fn(async (input: RequestInfo | URL, _init?: RequestInit) => {
    const path = String(input)
    if (path === '/api/v1/capabilities') {
      return new Response(JSON.stringify({ features: ['daemon'], job_types: jobTypes, resources: [] }), { status: 200 })
    }
    if (path === '/api/v1/cloud/connections') {
      return new Response(JSON.stringify({
        connections: [{ id: 7, kind: 'webdav', name: 'Archive', url: 'https://example.test', has_token: false, has_username: true, has_password: true, created_at: '', updated_at: '' }],
      }), { status: 200 })
    }
    if (path === '/api/v1/jobs') {
      return new Response(JSON.stringify({ job: { id: 23 }, duplicate: false }), { status: 202 })
    }
    throw new Error(`Unexpected request: ${path}`)
  })
  vi.stubGlobal('fetch', fetchMock)

  const [{ createApp, nextTick }, { createMemoryHistory, createRouter }, { default: OrganizeView }] = await Promise.all([
    import('vue'),
    import('vue-router'),
    import('./views/OrganizeView.vue'),
  ])
  const router = createRouter({
    history: createMemoryHistory(),
    routes: [
      { path: '/organize', component: { template: '<div />' } },
      { path: '/jobs/:id', component: { template: '<div />' } },
    ],
  })
  await router.push('/organize')
  await router.isReady()

  const host = document.createElement('div')
  document.body.appendChild(host)
  const app = createApp(OrganizeView).use(router)
  app.mount(host)
  cleanups.push(() => app.unmount())

  await vi.waitFor(() => expect(fetchMock).toHaveBeenCalledWith('/api/v1/capabilities', expect.anything()))
  await nextTick()
  await Promise.resolve()
  await nextTick()
  return { fetchMock, host, nextTick }
}

describe('OrganizeView storage capability', () => {
  it('keeps the local view and never requests cloud connections when storage organize is absent', async () => {
    const { fetchMock, host } = await mountOrganizeView(['organize'])

    expect(host.textContent).toContain('Paths and operation')
    expect(host.textContent).not.toContain('Cross-storage')
    expect(fetchMock.mock.calls.map(([input]) => String(input))).toEqual(['/api/v1/capabilities'])
  })

  it('shows storage organization, loads connections, and submits its confirmed typed payload', async () => {
    const { fetchMock, host, nextTick } = await mountOrganizeView(['organize', 'storage_organize'])
    await vi.waitFor(() => expect(fetchMock).toHaveBeenCalledWith('/api/v1/cloud/connections', expect.anything()))

    const storageTab = Array.from(host.querySelectorAll('button')).find(button => button.textContent?.includes('Cross-storage'))
    expect(storageTab).toBeDefined()
    storageTab?.click()
    await nextTick()

    const sourcePath = Array.from(host.querySelectorAll('label')).find(label => label.textContent?.includes('Source path'))?.querySelector('input')
    const targetConnection = Array.from(host.querySelectorAll('label')).find(label => label.textContent?.includes('Target connection'))?.querySelector('select')
    const confirmation = Array.from(host.querySelectorAll('label')).find(label => label.textContent?.includes('destination must verify'))?.querySelector('input')
    expect(host.textContent).toContain('Archive (WebDAV)')
    expect(sourcePath).toBeInstanceOf(HTMLInputElement)
    expect(targetConnection).toBeInstanceOf(HTMLSelectElement)
    expect(confirmation).toBeInstanceOf(HTMLInputElement)

    sourcePath!.value = ' D:/Incoming '
    sourcePath!.dispatchEvent(new Event('input', { bubbles: true }))
    targetConnection!.value = '7'
    targetConnection!.dispatchEvent(new Event('change', { bubbles: true }))
    confirmation!.checked = true
    confirmation!.dispatchEvent(new Event('change', { bubbles: true }))
    await nextTick()
    host.querySelector('form:last-of-type')!.dispatchEvent(new Event('submit', { bubbles: true, cancelable: true }))

    await vi.waitFor(() => expect(fetchMock).toHaveBeenCalledWith('/api/v1/jobs', expect.objectContaining({ method: 'POST' })))
    const request = fetchMock.mock.calls.find(([input]) => String(input) === '/api/v1/jobs')
    expect(JSON.parse(request?.[1]?.body as string)).toEqual({
      origin: 'manual',
      confirmed: true,
      job: {
        type: 'storage_organize',
        args: {
          source: { type: 'local', path: 'D:/Incoming' },
          target: { type: 'connection', connection_id: 7, path: '/' },
          mode: 'move',
          season_mode: true,
          mlip: false,
          remove_empty_dirs: true,
        },
      },
    })
  })
})
