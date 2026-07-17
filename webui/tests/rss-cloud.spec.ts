import { test, expect } from '@playwright/test'

test.describe('RSS and CloudDrive routes', () => {
  test('RSS table and subscription detail are responsive', async ({ page }) => {
    await page.route('/api/v1/rss/subscriptions', route => route.fulfill({ json: { subscriptions: [] } }))
    await page.goto('/rss')
    await expect(page.getByRole('status')).toContainText('Daemon online')
    await expect(page.getByRole('heading', { name: 'RSS subscriptions' })).toBeVisible()
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)).toBe(true)
    expect(await page.locator('main').boundingBox()).not.toBeNull()
    await expect(page.locator('body')).toHaveScreenshot('rss-mobile.png')
  })

  test('CloudDrive folder route has bounded layout and never stores entered secrets', async ({ page }) => {
    await page.route('/api/v1/cloud/connections', route => route.fulfill({ json: { connections: [] } }))
    await page.goto('/cloud')
    await expect(page.getByRole('status')).toContainText('Daemon online')
    expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)).toBe(true)
    await page.getByLabel('Token').fill('test-token')
    expect(await page.evaluate(() => Object.keys(localStorage))).toEqual([])
  })
})
