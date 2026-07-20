import { test, expect } from '@playwright/test'

test.describe('RSS and CloudDrive routes', () => {
  test.beforeEach(async ({ page }) => {
    await page.route('**/api/v1/health', route => route.fulfill({ json: { status: 'ok', version: 'test' } }))
    await page.route('**/api/v1/status', route => route.fulfill({ json: { uptime_seconds: 60, worker_state: 'idle', current_job_id: null, queue_counts: { queued: 0, running: 0, succeeded: 0, failed: 0, canceled: 0 }, database_path: 'test.db' } }))
    await page.route('**/api/v1/capabilities', route => route.fulfill({ json: { features: ['metadata', 'clouddrive'], job_types: ['organize', 'rss_poll', 'rss_poll_all', 'cloud_add_offline'], resources: ['rss_subscriptions', 'cloud_connections'] } }))
    await page.route('**/api/v1/rss/subscriptions', route => route.fulfill({ json: { subscriptions: [] } }))
    await page.route('**/api/v1/cloud/connections', route => route.fulfill({ json: { connections: [] } }))
  })

  test('RSS subscription page is responsive', async ({ page }) => {
    await page.goto('/rss')
    await expect(page.getByRole('status')).toContainText('Daemon online')
    await expect(page.getByRole('heading', { name: 'RSS subscriptions' })).toBeVisible()
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)).toBe(true)
    expect(await page.locator('main').boundingBox()).not.toBeNull()
    await expect(page.locator('body')).toHaveScreenshot('rss-mobile.png')
  })

  test('CloudDrive folder route has bounded layout and never stores entered secrets', async ({ page }) => {
    await page.goto('/cloud')
    await expect(page.getByRole('status')).toContainText('Daemon online')
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)).toBe(true)
    await page.getByLabel('Token').fill('test-token')
    expect(await page.evaluate(() => JSON.stringify(localStorage))).not.toContain('test-token')
  })
})
