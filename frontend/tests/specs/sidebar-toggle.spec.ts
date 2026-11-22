import type { Page } from '@playwright/test';
import { test, expect } from '../fixtures/api.fixture';

// SB: collapsible sidebar (>= md). Contract from the frontend agent: toggle button with aria-expanded +
// aria-controls="app-sidebar", .ffm-shell[data-sidebar], rail 220px / 64px, labels stay in the DOM
// (opacity 0) so link names are unchanged, localStorage 'ffm.sidebar', no transition under reduced motion.

const aside = (page: Page) => page.locator('#app-sidebar');
const toggle = (page: Page) => page.getByRole('button', { name: 'Toggle sidebar' });
const shell = (page: Page) => page.locator('.ffm-shell');
const width = (page: Page) => aside(page).evaluate((e) => e.getBoundingClientRect().width);
const NAV = ['Dashboard', 'Imports', 'Settings'];

test.describe('Shell > collapsible sidebar (SB)', () => {
  test.beforeEach(async ({ mockDashboard }) => {
    await mockDashboard();
  });

  test('SB1 toggle collapses and expands; links stay reachable by name', async ({ page }) => {
    await page.goto('/');
    await expect(toggle(page)).toHaveAttribute('aria-expanded', 'true');
    await expect(toggle(page)).toHaveAttribute('aria-controls', 'app-sidebar');
    await expect(shell(page)).toHaveAttribute('data-sidebar', 'expanded');
    await expect.poll(() => width(page)).toBe(220);

    await toggle(page).click();
    await expect(toggle(page)).toHaveAttribute('aria-expanded', 'false');
    await expect(shell(page)).toHaveAttribute('data-sidebar', 'collapsed');
    await expect.poll(() => width(page)).toBe(64);
    await expect
      .poll(() => page.locator('.ffm-content').evaluate((e) => getComputedStyle(e).paddingLeft))
      .toBe('64px');
    for (const name of NAV) {
      const link = aside(page).getByRole('link', { name });
      await expect(link, `${name} link keeps its accessible name`).toBeVisible();
      await expect(link).toHaveAttribute('title', name);
      await expect
        .poll(
          () => link.locator('.ffm-sidebar-label').evaluate((e) => getComputedStyle(e).opacity),
          { message: `${name} label hidden` },
        )
        .toBe('0');
    }
    // a collapsed rail still navigates
    await aside(page).getByRole('link', { name: 'Settings' }).click();
    await expect(page).toHaveURL(/\/settings$/);

    // keyboard: Enter on the focused toggle expands again
    await toggle(page).focus();
    await page.keyboard.press('Enter');
    await expect(toggle(page)).toHaveAttribute('aria-expanded', 'true');
    await expect(toggle(page), 'focus stays on the toggle after Enter').toBeFocused();
    await page.keyboard.press('Space');
    await expect(toggle(page)).toHaveAttribute('aria-expanded', 'false');
    await expect(toggle(page), 'focus stays on the toggle after Space').toBeFocused();
    await page.keyboard.press('Enter');
    await expect(toggle(page)).toHaveAttribute('aria-expanded', 'true');
    await expect.poll(() => width(page)).toBe(220);
    await expect(aside(page).getByRole('link', { name: 'Dashboard' })).not.toHaveAttribute('title');
    await expect
      .poll(() =>
        aside(page)
          .locator('.ffm-sidebar-label')
          .first()
          .evaluate((e) => getComputedStyle(e).opacity),
      )
      .toBe('1');
  });

  test('SB2 state persists across reload', async ({ page }) => {
    await page.goto('/');
    await toggle(page).click();
    await expect(shell(page)).toHaveAttribute('data-sidebar', 'collapsed');
    expect(await page.evaluate(() => localStorage.getItem('ffm.sidebar'))).toBe('collapsed');

    await page.reload();
    await expect(shell(page)).toHaveAttribute('data-sidebar', 'collapsed');
    await expect(toggle(page)).toHaveAttribute('aria-expanded', 'false');
    await expect.poll(() => width(page)).toBe(64);

    await toggle(page).click();
    expect(await page.evaluate(() => localStorage.getItem('ffm.sidebar'))).toBe('expanded');
    await page.reload();
    await expect(shell(page)).toHaveAttribute('data-sidebar', 'expanded');
    await expect(toggle(page)).toHaveAttribute('aria-expanded', 'true');
  });

  test('SB2b junk stored value falls back to expanded without crashing', async ({ page }) => {
    const errors: string[] = [];
    page.on('pageerror', (e) => errors.push(e.message));
    await page.addInitScript(() => localStorage.setItem('ffm.sidebar', '{not-a-state'));
    await page.goto('/');
    await expect(shell(page)).toHaveAttribute('data-sidebar', 'expanded');
    await expect(toggle(page)).toHaveAttribute('aria-expanded', 'true');
    await toggle(page).click();
    await expect(shell(page)).toHaveAttribute('data-sidebar', 'collapsed');
    expect(
      await page.evaluate(() => localStorage.getItem('ffm.sidebar')),
      'toggle overwrites the junk value',
    ).toBe('collapsed');
    expect(errors).toEqual([]);
  });

  test('SB2c persisted state paints at its width on load (no animate-on-load)', async ({
    page,
  }) => {
    // Runs before the app: store "collapsed", then record the rail width every frame and any rail transition.
    await page.addInitScript(() => {
      localStorage.setItem('ffm.sidebar', 'collapsed');
      const w: number[] = [];
      const runs: string[] = [];
      Object.assign(window, { __w: w, __runs: runs });
      document.addEventListener('transitionrun', (e) => {
        // Only layout properties: content cards may fade colours/borders on mount without moving the rail.
        const prop = (e as TransitionEvent).propertyName;
        if (
          (e.target as Element).closest?.('#app-sidebar, .ffm-content') &&
          /^(width|min-width|max-width|inline-size|padding(-.+)?|margin(-.+)?)$/.test(prop)
        )
          runs.push(prop);
      });
      const tick = () => {
        const a = document.getElementById('app-sidebar');
        if (a) w.push(a.getBoundingClientRect().width);
        if (performance.now() < 1500) requestAnimationFrame(tick);
      };
      requestAnimationFrame(tick);
    });
    await page.goto('/');
    await expect(shell(page)).toHaveAttribute('data-sidebar', 'collapsed');
    await page.waitForFunction(() => performance.now() >= 1500);
    const { w, runs } = await page.evaluate(() => {
      const rec = window as unknown as { __w: number[]; __runs: string[] };
      return { w: rec.__w, runs: rec.__runs };
    });
    expect(w.length, 'rail sampled from first paint').toBeGreaterThan(0);
    expect([...new Set(w)], 'every sampled width since first paint').toEqual([64]);
    expect(runs, 'no width/padding transition on load').toEqual([]);
    // Control: the same listener does see a transition on a real toggle (motion allowed).
    await toggle(page).click();
    await expect
      .poll(() => page.evaluate(() => (window as unknown as { __runs: string[] }).__runs.length))
      .toBeGreaterThan(0);
  });

  test('SB3 reduced motion: no transition, width snaps', async ({ page }) => {
    const durations = () =>
      page.evaluate(() =>
        ['.ffm-sidebar', '.ffm-content', '.ffm-sidebar-label'].map(
          (s) => getComputedStyle(document.querySelector(s)!).transitionDuration,
        ),
      );
    await page.goto('/');
    // Control: with motion allowed the rail does animate (otherwise the reduced check proves nothing).
    expect(
      (await durations()).some((d) => d !== '0s'),
      'transitions exist with motion allowed',
    ).toBe(true);

    await page.emulateMedia({ reducedMotion: 'reduce' });
    expect(await durations()).toEqual(['0s', '0s', '0s']);
    await toggle(page).click();
    expect(await width(page), 'width right after the click, no animation frame').toBe(64);
    // Content padding must not lag either (with motion it waits for the collapse to end).
    expect(
      await page.locator('.ffm-content').evaluate((e) => getComputedStyle(e).paddingLeft),
    ).toBe('64px');
  });

  test('SB4 below md the rail is hidden and the bottom nav is used', async ({ page }) => {
    await page.setViewportSize({ width: 390, height: 844 });
    await page.goto('/');
    await expect(aside(page)).toBeHidden();
    await expect(toggle(page)).toBeHidden();
    for (const name of NAV)
      await expect(page.getByRole('link', { name }).filter({ visible: true })).toHaveCount(1);
  });
});
