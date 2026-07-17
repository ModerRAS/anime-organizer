import { expect, test } from '@playwright/test'

for (const path of ['/', '/jobs', '/jobs/1', '/organize', '/scraper', '/torrent', '/aliases', '/about']) {
  test(`renders ${path} without horizontal overflow`, async ({ page }) => {
    const errors: string[] = []
    page.on('pageerror', (error) => errors.push(error.message))
    page.on('console', (message) => { if (message.type() === 'error') errors.push(message.text()) })
    page.on('requestfailed', (request) => errors.push(request.failure()?.errorText ?? `request failed: ${request.url()}`))
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
