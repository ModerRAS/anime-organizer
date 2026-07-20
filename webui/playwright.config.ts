import { defineConfig } from '@playwright/test'

const externalBaseUrl = process.env.WEBUI_BASE_URL

export default defineConfig({
  testDir: './tests',
  workers: 1,
  webServer: externalBaseUrl ? undefined : {
    command: 'npm run dev -- --host 127.0.0.1 --port 4173',
    url: 'http://127.0.0.1:4173',
    reuseExistingServer: true,
  },
  use: { baseURL: externalBaseUrl ?? 'http://127.0.0.1:4173' },
  projects: [
    { name: 'desktop', use: { viewport: { width: 1440, height: 900 } } },
    { name: 'mobile', use: { viewport: { width: 390, height: 844 } } },
  ],
})
