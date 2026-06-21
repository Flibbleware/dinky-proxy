import { expect, fullPageScreenshot, test } from '../tauri-fixture'

test('loads the settings page', async ({ page, pageUrl }) => {
  await page.goto(pageUrl)
  await expect(page.getByLabel('Host', { exact: true })).toBeVisible()
  await fullPageScreenshot(page, 'settings-default')
})

test('shows all basic fields on load', async ({ page, pageUrl }) => {
  await page.goto(pageUrl)

  // `exact` distinguishes the inputs from the adjacent "More information about …" help triggers.
  await expect(page.getByLabel('Host', { exact: true })).toBeVisible()
  await expect(page.getByLabel('Port')).toBeVisible()
  await expect(page.getByLabel('Username')).toBeVisible()
  await expect(page.getByLabel('Password', { exact: true })).toBeVisible()
  await expect(page.getByLabel('Domains')).toBeVisible()
})

test('shows validation error for empty required field', async ({ page, pageUrl }) => {
  await page.goto(pageUrl)

  const proxyHost = page.getByLabel('Host', { exact: true })
  await expect(proxyHost).toHaveValue('')

  // Save stays disabled until the form is dirty, so edit another field to enable submit
  // while leaving the required Host empty.
  await page.getByLabel('Port').fill('8081')
  await page.getByRole('button', { name: 'Save configuration' }).click()

  await expect(page.getByText('Proxy host is required')).toBeVisible()
  await expect(proxyHost).toHaveAttribute('aria-invalid', 'true')
  await fullPageScreenshot(page, 'settings-validation-errors')
})

test('reveals field hint tooltips on focus', async ({ page, pageUrl }) => {
  await page.goto(pageUrl)

  // Hints stay hidden until the help trigger is focused or hovered.
  await expect(page.getByRole('tooltip')).toHaveCount(0)

  await page.getByRole('button', { name: 'More information about Host' }).focus()
  await expect(page.getByRole('tooltip')).toHaveText('The remote proxy hostname / IP address')

  // Focusing the next trigger dismisses the previous tooltip.
  await page.getByRole('button', { name: 'More information about Password' }).focus()
  await expect(page.getByRole('tooltip')).toHaveText('Stored securely in the keychain')
})

test('toggles advanced settings', async ({ page, pageUrl }) => {
  await page.goto(pageUrl)

  // `exact` distinguishes each control from its adjacent "More information about …" help trigger.
  await expect(page.getByLabel('Proxy protocol', { exact: true })).not.toBeVisible()
  await expect(page.getByLabel('Local Server port', { exact: true })).not.toBeVisible()
  await expect(page.getByLabel('PAC server port', { exact: true })).not.toBeVisible()
  await expect(page.getByLabel('Network service', { exact: true })).not.toBeVisible()

  await page.getByRole('button', { name: 'Show advanced settings' }).click()

  await expect(page.getByLabel('Proxy protocol', { exact: true })).toBeVisible()
  await expect(page.getByLabel('Local Server port', { exact: true })).toBeVisible()
  await expect(page.getByLabel('PAC server port', { exact: true })).toBeVisible()
  await expect(page.getByLabel('Network service', { exact: true })).toBeVisible()
  await expect(page.getByRole('button', { name: 'Hide advanced settings' })).toBeVisible()
  await fullPageScreenshot(page, 'settings-advanced')
})
