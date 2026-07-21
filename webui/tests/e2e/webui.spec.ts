import { expect, test } from '@playwright/test'

test.beforeEach(async ({ page }) => {
  await page.route('**/api/v1/health', route => route.fulfill({ json: { status: 'ok', version: 'test' } }))
  await page.route('**/api/v1/status', route => route.fulfill({ json: { uptime_seconds: 60, worker_state: 'idle', current_job_id: null, queue_counts: { queued: 0, running: 0, succeeded: 0, failed: 0, canceled: 0 }, database_path: 'test.db' } }))
  await page.route('**/api/v1/capabilities', route => route.fulfill({ json: { features: ['metadata', 'clouddrive', 'scraper', 'torrent-scraper'], job_types: ['organize', 'scrape', 'match_aliases', 'torrent_scrape', 'build_bangumi_db', 'extract_aliases', 'merge_aliases', 'apply_matches', 'create_alias_issues', 'rss_poll', 'rss_poll_all', 'cloud_add_offline'], resources: ['rss_subscriptions', 'cloud_connections'] } }))
  await page.route('**/api/v1/jobs?*', route => route.fulfill({ json: { jobs: [] } }))
  await page.route('**/api/v1/jobs/*/logs?*', route => route.fulfill({ json: { logs: [] } }))
})

for (const path of ['/', '/jobs', '/jobs/1', '/organize', '/scraper', '/torrent', '/aliases', '/about']) {
  test(`renders ${path} without horizontal overflow`, async ({ page }) => {
    const errors: string[] = []
    page.on('pageerror', (error) => errors.push(error.message))
    page.on('console', (message) => { if (message.type() === 'error') errors.push(message.text()) })
    page.on('requestfailed', (request) => errors.push(request.failure()?.errorText ?? `request failed: ${request.url()}`))
    if (path === '/jobs/1') await page.route('**/api/v1/jobs/1', route => route.fulfill({ json: { ...job, id: 1 } }))
    await page.goto(path)
    await expect(page.getByRole('status')).toContainText('Daemon online')
    await expect(page.locator('main')).toBeVisible()
    expect(errors).toEqual([])
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)).toBe(true)
    const main = page.locator('main')
    expect(await main.boundingBox()).not.toBeNull()
    await expect(main).toHaveCSS('min-width', '0px')
    for (const region of ['header', 'aside', 'main']) {
      const box = await page.locator(region).boundingBox()
      expect(box?.width).toBeGreaterThan(0)
      expect(box?.height).toBeGreaterThan(0)
    }
    const screenshotName = path === '/' ? 'dashboard' : path.slice(1).replaceAll('/', '-')
    await page.screenshot({ path: `test-results/${screenshotName}.png`, fullPage: true })
  })
}

const job = {
  id: 9001,
  idempotency_key: null,
  origin: 'manual',
  kind: 'cloud_add_offline',
  resource_key: null,
  request: { type: 'cloud_add_offline', args: {} },
  state: 'succeeded',
  priority: 200,
  attempts: 1,
  progress_current: null,
  progress_total: null,
  progress_message: null,
  result: null,
  error: null,
  created_at: '1784422777',
  started_at: '1784422777',
  finished_at: '1784422800',
}

test('formats legacy timestamps and persists the Chinese locale', async ({ page }, testInfo) => {
  await page.unroute('**/api/v1/jobs?*')
  await page.route('**/api/v1/jobs?*', route => route.fulfill({ json: { jobs: [job] } }))
  await page.goto('/jobs')
  await expect(page.locator('main')).not.toContainText('Invalid Date')
  await page.getByRole('button', { name: '中文' }).click()
  await expect(page.getByRole('heading', { name: '任务', exact: true })).toBeVisible()
  await expect(page.locator('html')).toHaveAttribute('lang', 'zh-CN')
  await page.reload()
  await expect(page.getByRole('heading', { name: '任务', exact: true })).toBeVisible()
  await expect(page.getByRole('status')).toContainText('Daemon 在线')
  await expect(page.locator('main')).not.toContainText('Invalid Date')
  await page.screenshot({ path: `test-results/jobs-chinese-${testInfo.project.name}.png`, fullPage: true })
})

test('keeps page sections padded on desktop and mobile', async ({ page }) => {
  await page.goto('/')
  const section = page.locator('.section-block').first()
  await expect(section).toBeVisible()
  const spacing = await section.evaluate(element => {
    const style = getComputedStyle(element)
    const box = element.getBoundingClientRect()
    const heading = element.querySelector('.section-heading')!.getBoundingClientRect()
    return {
      border: style.borderLeftWidth,
      leftInset: Math.round(heading.left - box.left),
      rightPadding: parseFloat(style.paddingRight),
    }
  })
  expect(spacing.border).toBe('1px')
  expect(spacing.leftInset).toBeGreaterThanOrEqual(16)
  expect(spacing.rightPadding).toBeGreaterThanOrEqual(16)

  await page.goto('/jobs')
  const emptyCell = page.locator('.empty-cell')
  await expect(emptyCell).toBeVisible()
  const mainBox = await page.locator('main').boundingBox()
  const emptyBox = await emptyCell.boundingBox()
  expect(emptyBox!.x).toBeGreaterThanOrEqual(mainBox!.x)
  expect(emptyBox!.x).toBeLessThan(mainBox!.x + mainBox!.width)
})

test('shares capabilities across route navigation and keeps shell scroll regions independent', async ({ page }) => {
  let capabilityCalls = 0
  await page.unroute('**/api/v1/capabilities')
  await page.route('**/api/v1/capabilities', async route => {
    capabilityCalls++
    await new Promise(resolve => setTimeout(resolve, 300))
    await route.fulfill({ json: { features: ['metadata', 'clouddrive'], job_types: ['organize', 'scrape', 'match_aliases', 'torrent_scrape', 'build_bangumi_db'], resources: ['rss_subscriptions'] } })
  })
  await page.goto('/')
  const menu = page.getByRole('button', { name: 'Open navigation' })
  if (await menu.isVisible()) await menu.click()
  await page.getByRole('link', { name: 'Scraper' }).click()
  await expect(page).toHaveURL(/\/scraper$/)
  if (await menu.isVisible()) await menu.click()
  await page.getByRole('link', { name: 'Torrents' }).click()
  await expect(page).toHaveURL(/\/torrent$/)
  expect(capabilityCalls).toBe(1)
  expect(await page.evaluate(() => ({
    body: getComputedStyle(document.body).overflow,
    sidebar: getComputedStyle(document.querySelector('aside')!).overflowY,
    main: getComputedStyle(document.querySelector('main')!).overflowY,
  }))).toEqual({ body: 'hidden', sidebar: 'auto', main: 'auto' })
})

test('shows job finish time, duration, and incremental logs', async ({ page }) => {
  await page.route('**/api/v1/jobs/9001', route => route.fulfill({ json: { ...job, state: 'running', finished_at: null } }))
  await page.unroute('**/api/v1/jobs/*/logs?*')
  let logCalls = 0
  await page.route('**/api/v1/jobs/9001/logs?*', route => {
    logCalls++
    return route.fulfill({ json: { logs: logCalls === 1
      ? [{ id: 1, level: 'info', message: 'Worker started the job', created_at: '2026-07-20T12:00:00Z' }]
      : [{ id: 2, level: 'info', message: 'Organized episode 1', created_at: '2026-07-20T12:00:02Z' }] } })
  })
  await page.goto('/jobs/9001')
  await expect(page.getByRole('log')).toContainText('Worker started the job')
  await expect(page.getByRole('log')).toContainText('Organized episode 1', { timeout: 5000 })
  await expect(page.getByText('Duration', { exact: true })).toBeVisible()
})

test('ignores an older jobs poll that finishes late', async ({ page }) => {
  await page.unroute('**/api/v1/jobs?*')
  let calls = 0
  await page.route('**/api/v1/jobs?*', async route => {
    const call = ++calls
    if (call === 2) await new Promise(resolve => setTimeout(resolve, 2500))
    await route.fulfill({ json: { jobs: [{ ...job, state: call >= 3 ? 'succeeded' : 'queued' }] } })
  })
  await page.goto('/jobs')
  await expect(page.locator('.state')).toContainText('queued')
  await expect(page.locator('.state')).toContainText('succeeded', { timeout: 6000 })
  await page.waitForTimeout(1000)
  await expect(page.locator('.state')).toContainText('succeeded')
})

test('restarts pagination after a burst larger than one page', async ({ page }) => {
  await page.unroute('**/api/v1/jobs?*')
  let firstPage = true
  await page.route('**/api/v1/jobs?*', route => {
    const beforeId = new URL(route.request().url()).searchParams.get('before_id')
    const ids = beforeId === '301'
      ? Array.from({ length: 100 }, (_, index) => 300 - index)
      : firstPage
        ? Array.from({ length: 100 }, (_, index) => 200 - index)
        : Array.from({ length: 100 }, (_, index) => 400 - index)
    firstPage = false
    return route.fulfill({ json: { jobs: ids.map(id => ({ ...job, id })) } })
  })
  await page.goto('/jobs')
  await expect(page.getByRole('link', { name: '#200', exact: true })).toBeVisible()
  await expect(page.getByRole('link', { name: '#400', exact: true })).toBeVisible({ timeout: 5000 })
  await page.getByRole('button', { name: 'Load older' }).click()
  await expect(page.getByRole('link', { name: '#300', exact: true })).toBeVisible()
})

test('redirects an unavailable capability route', async ({ page }) => {
  await page.unroute('**/api/v1/capabilities')
  await page.route('**/api/v1/capabilities', route => route.fulfill({ json: { features: ['metadata'], job_types: ['organize'], resources: [] } }))
  await page.goto('/cloud')
  await expect(page).toHaveURL(/\/about\?unavailable=clouddrive/)
  await expect(page.getByRole('alert')).toContainText('current build')
})

test('reloads RSS detail when the route ID changes', async ({ page }) => {
  let subscriptionCalls = 0
  await page.route('**/api/v1/rss/subscriptions', async route => {
    subscriptionCalls++
    if (subscriptionCalls === 1) await new Promise(resolve => setTimeout(resolve, 300))
    await route.fulfill({ json: { subscriptions: [
    { id: 1, url: 'https://example.test/one.xml', filter_regex: null, target_folder: '/one', interval_secs: 300, enabled: true, last_checked_at: null, connection_id: 1 },
    { id: 2, url: 'https://example.test/two.xml', filter_regex: null, target_folder: '/two', interval_secs: 300, enabled: true, last_checked_at: null, connection_id: 1 },
  ] } })
  })
  await page.route('**/api/v1/rss/processed?*', route => route.fulfill({ json: { items: [] } }))
  await page.route('**/api/v1/rss/download-tasks?*', route => route.fulfill({ json: { tasks: [] } }))
  await page.goto('/rss/1')
  await expect.poll(() => subscriptionCalls).toBe(1)
  await page.evaluate(() => {
    history.pushState({}, '', '/rss/2')
    window.dispatchEvent(new PopStateEvent('popstate'))
  })
  await expect(page.locator('main')).toContainText('two.xml')
  await page.waitForTimeout(350)
  await expect(page.locator('main')).not.toContainText('one.xml')
})
