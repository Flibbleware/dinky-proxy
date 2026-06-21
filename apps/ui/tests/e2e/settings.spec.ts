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
  await expect(page.getByLabel('Port', { exact: true })).toBeVisible()
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
  await page.getByLabel('Port', { exact: true }).fill('8081')
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

test('swaps the container to advanced settings', async ({ page, pageUrl }) => {
  await page.goto(pageUrl)

  // Basic fields are shown first; advanced fields are not rendered yet.
  await expect(page.getByLabel('Host', { exact: true })).toBeVisible()
  await expect(page.getByLabel('Proxy protocol', { exact: true })).not.toBeVisible()

  await page.getByRole('button', { name: 'Advanced settings' }).click()

  // Advanced fields replace the basic ones in the same container, and the toggle flips.
  await expect(page.getByLabel('Proxy protocol', { exact: true })).toBeVisible()
  await expect(page.getByLabel('Local Server port', { exact: true })).toBeVisible()
  await expect(page.getByLabel('PAC server port', { exact: true })).toBeVisible()
  await expect(page.getByLabel('Network service', { exact: true })).toBeVisible()
  await expect(page.getByLabel('Host', { exact: true })).not.toBeVisible()
  await expect(page.getByRole('button', { name: 'Basic settings' })).toBeVisible()
  await fullPageScreenshot(page, 'settings-advanced')
})

test('returns to basic settings with the toggle', async ({ page, pageUrl }) => {
  await page.goto(pageUrl)

  await page.getByRole('button', { name: 'Advanced settings' }).click()
  await expect(page.getByLabel('Proxy protocol', { exact: true })).toBeVisible()

  await page.getByRole('button', { name: 'Basic settings' }).click()
  await expect(page.getByLabel('Host', { exact: true })).toBeVisible()
  await expect(page.getByLabel('Proxy protocol', { exact: true })).not.toBeVisible()
})

test('adds a domain as a pill on Enter', async ({ page, pageUrl }) => {
  await page.goto(pageUrl)

  const domainsInput = page.getByLabel('Domains')
  await domainsInput.fill('example.com')
  await domainsInput.press('Enter')

  await expect(page.getByRole('button', { name: 'Remove example.com' })).toBeVisible()
  await expect(domainsInput).toHaveValue('')
})

test('removes a domain pill', async ({ page, pageUrl }) => {
  await page.goto(pageUrl)

  const domainsInput = page.getByLabel('Domains')
  await domainsInput.fill('example.com')
  await domainsInput.press('Enter')
  await expect(page.getByRole('button', { name: 'Remove example.com' })).toBeVisible()

  await page.getByRole('button', { name: 'Remove example.com' }).click()
  await expect(page.getByRole('button', { name: 'Remove example.com' })).not.toBeVisible()
})

test('does not add duplicate domains', async ({ page, pageUrl }) => {
  await page.goto(pageUrl)

  const domainsInput = page.getByLabel('Domains')
  await domainsInput.fill('example.com')
  await domainsInput.press('Enter')
  await domainsInput.fill('example.com')
  await domainsInput.press('Enter')

  await expect(page.getByRole('button', { name: 'Remove example.com' })).toHaveCount(1)
})

test('adds multiple domains as pills', async ({ page, pageUrl }) => {
  await page.goto(pageUrl)

  const domainsInput = page.getByLabel('Domains')
  await domainsInput.fill('example.com')
  await domainsInput.press('Enter')
  await domainsInput.fill('*.internal.company')
  await domainsInput.press('Enter')

  await expect(page.getByRole('button', { name: 'Remove example.com' })).toBeVisible()
  await expect(page.getByRole('button', { name: 'Remove *.internal.company' })).toBeVisible()
  await fullPageScreenshot(page, 'settings-domains-pills')
})
