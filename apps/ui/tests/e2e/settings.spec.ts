import { test, expect } from '../tauri-fixture'

test('loads the settings page', async ({ page, pageUrl }) => {
  await page.goto(pageUrl)
  await expect(page.getByText('DinkyProxy')).toBeVisible()
})

test('shows all basic fields on load', async ({ page, pageUrl }) => {
  await page.goto(pageUrl)

  await expect(page.getByLabel('Proxy host')).toBeVisible()
  await expect(page.getByLabel('Port')).toBeVisible()
  await expect(page.getByLabel('Username')).toBeVisible()
  await expect(page.getByLabel('Password')).toBeVisible()
  await expect(page.getByLabel('Use proxy for domains')).toBeVisible()
})

test('shows validation error for empty required field', async ({ page, pageUrl }) => {
  await page.goto(pageUrl)

  const proxyHost = page.getByLabel('Proxy host')
  await expect(proxyHost).toHaveValue('xx.xx.xx.xx')
  await proxyHost.clear()
  await page.getByRole('button', { name: 'Save configuration' }).click()

  await expect(page.getByText('Proxy host is required')).toBeVisible()
  await expect(page.getByLabel('Proxy host')).toHaveAttribute('aria-invalid', 'true')
})

test('toggles advanced settings', async ({ page, pageUrl }) => {
  await page.goto(pageUrl)

  await expect(page.getByLabel('Proxy protocol')).not.toBeVisible()
  await expect(page.getByLabel('Local Server port')).not.toBeVisible()
  await expect(page.getByLabel('PAC server port')).not.toBeVisible()
  await expect(page.getByLabel('Network service')).not.toBeVisible()

  await page.getByRole('button', { name: 'Show advanced settings' }).click()

  await expect(page.getByLabel('Proxy protocol')).toBeVisible()
  await expect(page.getByLabel('Local Server port')).toBeVisible()
  await expect(page.getByLabel('PAC server port')).toBeVisible()
  await expect(page.getByLabel('Network service')).toBeVisible()
  await expect(page.getByRole('button', { name: 'Hide advanced settings' })).toBeVisible()
})
