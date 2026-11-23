import type { Page } from '@playwright/test';
import { test, expect } from '../fixtures/api.fixture';
import {
  mockSettings,
  mockTags,
  mockCategories,
  mockBudgets,
  mockCounterpartyResults,
} from '../mocks';

// RF-08 v0.11: each missing movement may carry `suggested_counterparty` (learned in the app, or from
// Firefly III history). It pre-fills the choice, is marked as a suggestion, and never creates anything
// by itself: the user can change it, clear it, or just press Create.
type Suggestion = {
  ff3_id?: number;
  name?: string;
  cash: boolean; // always present; ff3_id and name are omitted when there is no value
  source: 'learned' | 'history';
};

const mv = (id: number, description: string, amount_cents: number, s?: Suggestion | null) => ({
  id,
  line_no: id - 9,
  date: '2026-08-02',
  amount_cents,
  description,
  status: 'missing',
  ...(s === undefined ? {} : { suggested_counterparty: s }),
});

const LEARNED: Suggestion = { ff3_id: 51, name: 'Supermarket', cash: false, source: 'learned' };
const HISTORY: Suggestion = { ff3_id: 52, name: 'Pizza Place', cash: false, source: 'history' };
const BOOK = mv(12, 'Book', -9900, LEARNED);
const SHOES = mv(15, 'Shoes', -9900, HISTORY);
const SALARY = mv(13, 'Salary', 250000, {
  ff3_id: 61,
  name: 'Employer',
  cash: false,
  source: 'learned',
});
const GIFT = mv(14, 'Gift', -5000, { name: 'Acme Motors', cash: false, source: 'history' });
const COFFEE = mv(16, 'Coffee', -300, { cash: true, source: 'learned' });
const PLAIN = mv(17, 'Plain', -100, null);

const T = {
  en: {
    learned: 'Suggested (learned)',
    history: 'Suggested (Firefly III history)',
    clear: 'Clear suggestion',
    hint: (n: string) => `Suggested: ${n}`,
    edit: 'Edit',
    dest: 'Destination account',
    source: 'Source account',
    sheet: 'Edit movement',
    cash: 'Cash',
    create: /Create in Firefly III \(\d\)/,
    confirm: /^Create$/,
    tab: /Missing/,
  },
  es: {
    learned: 'Sugerida (aprendida)',
    history: 'Sugerida (historial de Firefly III)',
    clear: 'Quitar sugerencia',
    hint: (n: string) => `Sugerida: ${n}`,
    edit: 'Editar',
    dest: 'Cuenta destino',
    source: 'Cuenta origen',
    sheet: 'Editar movimiento',
    cash: 'Efectivo',
    create: /Crear en Firefly III \(\d\)/,
    confirm: /^Crear$/,
    tab: /Faltantes/,
  },
  fr: {
    learned: 'Suggérée (apprise)',
    history: 'Suggérée (historique Firefly III)',
    clear: 'Retirer la suggestion',
    hint: (n: string) => `Suggérée : ${n}`,
    edit: 'Modifier',
    dest: 'Compte de destination',
    source: 'Compte source',
    sheet: 'Modifier le mouvement',
    cash: 'Espèces',
    create: /Créer dans Firefly III \(\d\)/,
    confirm: /^Créer$/,
    tab: /Manquants/,
  },
} as const;
type L = (typeof T)[keyof typeof T];

const row = (page: Page, text: string) => page.getByRole('row').filter({ hasText: text });
const sheet = (page: Page, l: L = T.en) => page.getByRole('dialog', { name: l.sheet });
const combo = (page: Page, label: string, l: L = T.en) =>
  sheet(page, l).getByRole('combobox', { name: label });

async function openSheet(page: Page, text: string, l: L = T.en) {
  await row(page, text).getByRole('button', { name: l.edit, exact: true }).click();
  await expect(sheet(page, l)).toBeVisible();
}

/** Selects the rows and presses Create, without opening any sheet. */
async function createRows(
  page: Page,
  applied: { movements: Record<string, unknown>[] }[],
  texts: string[],
  l: L = T.en,
) {
  for (const t of texts) await row(page, t).getByRole('checkbox').check();
  await page.getByRole('button', { name: l.create }).click();
  await page.getByRole('dialog').getByRole('button', { name: l.confirm }).click();
  await expect.poll(() => applied.length).toBe(1);
  return applied[0].movements;
}

/** No counterparty chosen: created as Cash by the backend, but nothing is sent nor learned. */
const expectNothingChosen = (m: Record<string, unknown>) => {
  for (const k of ['counterparty_ff3_id', 'counterparty_name', 'counterparty_cash'])
    expect(m, k).not.toHaveProperty(k);
};
/** The explicit Cash choice (RF-08 v0.11): counterparty_cash alone. */
const expectCash = (m: Record<string, unknown>) => {
  expect(m).toMatchObject({ counterparty_cash: true });
  for (const k of ['counterparty_ff3_id', 'counterparty_name']) expect(m, k).not.toHaveProperty(k);
};

const byId = (ms: Record<string, unknown>[], id: number) => ms.find((m) => m.movement_id === id)!;

test.describe('Results > counterparty suggestions', () => {
  test.beforeEach(async ({ page, mockImportsPage }) => {
    await mockImportsPage();
    await mockSettings(page);
    await mockTags(page);
    await mockCategories(page);
    await mockBudgets(page);
  });

  test('G1 a learned suggestion pre-fills the sheet with the «learned» mark', async ({ page }) => {
    await mockCounterpartyResults(page, [BOOK]);
    await page.goto('/results/1');
    await openSheet(page, 'Book');
    await expect(combo(page, T.en.dest)).toHaveValue(/Supermarket/);
    await expect(sheet(page).getByText(T.en.learned, { exact: true })).toBeVisible();
    await expect(sheet(page).getByText(T.en.history, { exact: true })).toHaveCount(0);
  });

  test('G2 a Firefly III history suggestion carries the «history» mark', async ({ page }) => {
    await mockCounterpartyResults(page, [SHOES]);
    await page.goto('/results/1');
    await openSheet(page, 'Shoes');
    await expect(combo(page, T.en.dest)).toHaveValue(/Pizza Place/);
    await expect(sheet(page).getByText(T.en.history, { exact: true })).toBeVisible();
    await expect(sheet(page).getByText(T.en.learned, { exact: true })).toHaveCount(0);
  });

  test('G3 money in: the source account is pre-filled', async ({ page }) => {
    await mockCounterpartyResults(page, [SALARY]);
    await page.goto('/results/1');
    await openSheet(page, 'Salary');
    await expect(combo(page, T.en.source)).toHaveValue(/Employer/);
    await expect(sheet(page).getByText(T.en.learned, { exact: true })).toBeVisible();
  });

  test('G4 a name-only suggestion pre-fills «new account» and a Cash one pre-fills Cash', async ({
    page,
  }) => {
    await mockCounterpartyResults(page, [GIFT, COFFEE]);
    await page.goto('/results/1');
    await openSheet(page, 'Gift');
    await expect(combo(page, T.en.dest)).toHaveValue(/Acme Motors/);
    await expect(sheet(page).getByText(T.en.history, { exact: true })).toBeVisible();
    await page.keyboard.press('Escape');
    await openSheet(page, 'Coffee');
    await expect(combo(page, T.en.dest)).toHaveValue(/Cash/);
    await expect(sheet(page).getByText(T.en.learned, { exact: true })).toBeVisible();
  });

  test('G5 null (or absent) means no hint: empty sheet, no mark, no clear button', async ({
    page,
  }) => {
    await mockCounterpartyResults(page, [BOOK, PLAIN, mv(18, 'Absent', -100)]);
    await page.goto('/results/1');
    // a suggested row first: its mark must not leak into the next sheets
    await openSheet(page, 'Book');
    await expect(sheet(page).getByText(T.en.learned, { exact: true })).toBeVisible();
    await page.keyboard.press('Escape');
    await expect(sheet(page)).toHaveCount(0);
    for (const name of ['Plain', 'Absent']) {
      await expect(row(page, name)).not.toContainText(/Suggested/);
      await openSheet(page, name);
      await expect(combo(page, T.en.dest)).toHaveValue(/Cash/);
      await expect(sheet(page).getByText(/Suggested/)).toHaveCount(0);
      await expect(sheet(page).getByRole('button', { name: T.en.clear })).toHaveCount(0);
      await page.keyboard.press('Escape');
      await expect(sheet(page)).toHaveCount(0);
    }
  });

  test('G6 the Missing row shows a «Suggested: <account>» hint only when there is one', async ({
    page,
  }) => {
    await mockCounterpartyResults(page, [BOOK, GIFT, COFFEE, PLAIN]);
    await page.goto('/results/1');
    await expect(row(page, 'Book').getByText(T.en.hint('Supermarket'))).toBeVisible();
    await expect(row(page, 'Gift').getByText(T.en.hint('Acme Motors'))).toBeVisible();
    await expect(row(page, 'Coffee').getByText(T.en.hint(T.en.cash))).toBeVisible();
    await expect(row(page, 'Plain')).not.toContainText(/Suggested/);
  });

  test('G7 the confirm dialog shows the suggested account of each row, no sheet opened', async ({
    page,
  }) => {
    await mockCounterpartyResults(page, [BOOK, SALARY, PLAIN]);
    await page.goto('/results/1');
    await page.getByRole('checkbox', { name: 'Select all' }).check();
    await page.getByRole('button', { name: 'Create in Firefly III (3)' }).click();
    const dlg = page.getByRole('dialog');
    await expect(dlg.getByText('Withdrawal · Supermarket')).toBeVisible();
    await expect(dlg.getByText(T.en.learned)).toHaveCount(2); // Book and Salary; Plain has none
    await expect(dlg.getByText('Deposit · Employer')).toBeVisible();
    await expect(dlg.getByText('Withdrawal · Cash')).toBeVisible();
  });

  test.describe('Create without opening the sheet sends the suggestion', () => {
    test('G8 account id', async ({ page }) => {
      const applied = await mockCounterpartyResults(page, [BOOK]);
      await page.goto('/results/1');
      const [m] = await createRows(page, applied, ['Book']);
      expect(m).toMatchObject({ movement_id: 12, counterparty_ff3_id: 51 });
      expect(m).not.toHaveProperty('counterparty_name');
      expect(m).not.toHaveProperty('counterparty_cash');
    });

    test('G9 name only: counterparty_name and no id', async ({ page }) => {
      const applied = await mockCounterpartyResults(page, [GIFT]);
      await page.goto('/results/1');
      const [m] = await createRows(page, applied, ['Gift']);
      expect(m).toMatchObject({ movement_id: 14, counterparty_name: 'Acme Motors' });
      expect(m).not.toHaveProperty('counterparty_ff3_id');
      expect(m).not.toHaveProperty('counterparty_cash');
    });

    test('G10 a Cash suggestion sends counterparty_cash and nothing else', async ({ page }) => {
      const applied = await mockCounterpartyResults(page, [COFFEE]);
      await page.goto('/results/1');
      await openSheet(page, 'Coffee');
      await expect(sheet(page).getByText(T.en.learned, { exact: true })).toBeVisible();
      await page.keyboard.press('Escape');
      await expect(sheet(page)).toHaveCount(0);
      const [m] = await createRows(page, applied, ['Coffee']);
      expect(m).toMatchObject({ movement_id: 16 });
      expectCash(m);
    });

    test('G11 null: nothing sent (not even counterparty_cash), even next to a suggested row', async ({
      page,
    }) => {
      const applied = await mockCounterpartyResults(page, [PLAIN, BOOK]);
      await page.goto('/results/1');
      const ms = await createRows(page, applied, ['Plain', 'Book']);
      expect(byId(ms, 12)).toMatchObject({ counterparty_ff3_id: 51 });
      expectNothingChosen(byId(ms, 17));
    });

    test('G12 opening the sheet and saving untouched keeps the suggestion', async ({ page }) => {
      const applied = await mockCounterpartyResults(page, [BOOK]);
      await page.goto('/results/1');
      await openSheet(page, 'Book');
      await sheet(page).getByRole('button', { name: 'Save' }).click();
      await expect(sheet(page)).toHaveCount(0);
      const [m] = await createRows(page, applied, ['Book']);
      expect(m).toMatchObject({ movement_id: 12, counterparty_ff3_id: 51 });
    });

    test('G13 the suggestion alone does not create anything (O4/O5)', async ({ page }) => {
      const applied = await mockCounterpartyResults(page, [BOOK]);
      await page.goto('/results/1');
      await openSheet(page, 'Book');
      await expect(sheet(page).getByText(T.en.learned, { exact: true })).toBeVisible();
      await page.keyboard.press('Escape');
      await expect(row(page, 'Book').getByRole('checkbox')).not.toBeChecked();
      expect(applied).toHaveLength(0);
    });
  });

  test('G14 «Clear suggestion» empties the choice, drops the mark (nothing chosen, not Cash), and Create sends no account', async ({
    page,
  }) => {
    const applied = await mockCounterpartyResults(page, [BOOK]);
    await page.goto('/results/1');
    await openSheet(page, 'Book');
    await sheet(page).getByRole('button', { name: T.en.clear }).click();
    await expect(combo(page, T.en.dest)).toHaveValue('');
    await expect(sheet(page).getByText(/Suggested/)).toHaveCount(0);
    await expect(sheet(page).getByRole('button', { name: T.en.clear })).toHaveCount(0);
    await sheet(page).getByRole('button', { name: 'Save' }).click();
    await expect(sheet(page)).toHaveCount(0);
    await expect(row(page, 'Book')).not.toContainText(/Suggested/);
    // reopening does not bring the suggestion back
    await openSheet(page, 'Book');
    await expect(combo(page, T.en.dest)).toHaveValue('');
    await expect(sheet(page).getByText(/Suggested/)).toHaveCount(0);
    await sheet(page).getByRole('button', { name: 'Save' }).click();
    const [m] = await createRows(page, applied, ['Book']);
    expectNothingChosen(m); // cleared is not the explicit Cash: created as Cash, not learned
  });

  test('G15 changing the account sends the user choice and drops the mark', async ({ page }) => {
    const applied = await mockCounterpartyResults(page, [BOOK]);
    await page.goto('/results/1');
    await openSheet(page, 'Book');
    await expect(combo(page, T.en.dest)).toHaveValue(/Supermarket/);
    await expect(sheet(page).getByText(T.en.learned, { exact: true })).toBeVisible();
    await combo(page, T.en.dest).click();
    await page.getByRole('option', { name: /Car Loan/ }).click();
    await expect(combo(page, T.en.dest)).toHaveValue(/Car Loan/);
    await expect(sheet(page).getByText(/Suggested/)).toHaveCount(0);
    await sheet(page).getByRole('button', { name: 'Save' }).click();
    const [m] = await createRows(page, applied, ['Book']);
    expect(m).toMatchObject({ movement_id: 12, counterparty_ff3_id: 71 });
  });

  test('G16 choosing Cash explicitly over a suggestion sends no account', async ({ page }) => {
    const applied = await mockCounterpartyResults(page, [BOOK]);
    await page.goto('/results/1');
    await openSheet(page, 'Book');
    await expect(sheet(page).getByText(T.en.learned, { exact: true })).toBeVisible();
    await combo(page, T.en.dest).click();
    await page.getByRole('option', { name: /Cash/ }).click();
    await expect(sheet(page).getByText(/Suggested/)).toHaveCount(0);
    await sheet(page).getByRole('button', { name: 'Save' }).click();
    const [m] = await createRows(page, applied, ['Book']);
    expect(m).not.toHaveProperty('counterparty_ff3_id');
    expect(m).not.toHaveProperty('counterparty_name');
  });

  test('G23 saving a row with no suggestion without touching the account sends no Cash', async ({
    page,
  }) => {
    const applied = await mockCounterpartyResults(page, [PLAIN, BOOK]);
    await page.goto('/results/1');
    await openSheet(page, 'Plain');
    // the sheet shows Cash as the default, but only picking it in the combobox is explicit
    await expect(combo(page, T.en.dest)).toHaveValue(/Cash/);
    await sheet(page).getByRole('textbox', { name: 'Notes' }).fill('only a note');
    await sheet(page).getByRole('button', { name: 'Save' }).click();
    await expect(sheet(page)).toHaveCount(0);
    const ms = await createRows(page, applied, ['Plain', 'Book']);
    expect(byId(ms, 12)).toMatchObject({ counterparty_ff3_id: 51 });
    expect(byId(ms, 17)).toMatchObject({ notes: 'only a note' });
    expectNothingChosen(byId(ms, 17));
  });

  test('G24 saving a Cash suggestion untouched keeps counterparty_cash', async ({ page }) => {
    const applied = await mockCounterpartyResults(page, [COFFEE]);
    await page.goto('/results/1');
    await openSheet(page, 'Coffee');
    await expect(sheet(page).getByText(T.en.learned, { exact: true })).toBeVisible();
    await sheet(page).getByRole('button', { name: 'Save' }).click();
    await expect(sheet(page)).toHaveCount(0);
    const [m] = await createRows(page, applied, ['Coffee']);
    expectCash(m);
  });

  // RF-08 v0.11: a learned account may be gone from the cached lists (the id is stale): show its name
  test('G22 a suggested id missing from the account lists is shown by name, never as #id', async ({
    page,
  }) => {
    const OLD = mv(19, 'Old', -700, {
      ff3_id: 999,
      name: 'Old Shop',
      cash: false,
      source: 'learned',
    });
    const applied = await mockCounterpartyResults(page, [OLD]);
    await page.goto('/results/1');
    await expect(row(page, 'Old').getByText(T.en.hint('Old Shop'))).toBeVisible();
    await openSheet(page, 'Old');
    await expect(combo(page, T.en.dest)).toHaveValue(/Old Shop/);
    await expect(combo(page, T.en.dest)).not.toHaveValue(/#999/);
    await expect(sheet(page).getByText(T.en.learned, { exact: true })).toBeVisible();
    await page.keyboard.press('Escape');
    await row(page, 'Old').getByRole('checkbox').check();
    await page.getByRole('button', { name: T.en.create }).click();
    const dlg = page.getByRole('dialog');
    // the type is decided by the backend and unknown here: only the name and the mark
    await expect(dlg.getByText('Old Shop · Suggested (learned)')).toBeVisible();
    await expect(dlg).not.toContainText('#999');
    await expect(dlg).not.toContainText(/Withdrawal|Transfer/);
    await dlg.getByRole('button', { name: T.en.confirm }).click();
    await expect.poll(() => applied.length).toBe(1);
    expect(applied[0].movements[0]).toMatchObject({ movement_id: 19, counterparty_ff3_id: 999 });
    expect(applied[0].movements[0]).not.toHaveProperty('counterparty_name');
  });

  test.describe('bulk', () => {
    test('G17 Create on several rows sends each row its own suggestion', async ({ page }) => {
      const applied = await mockCounterpartyResults(page, [BOOK, SHOES, GIFT, COFFEE, PLAIN]);
      await page.goto('/results/1');
      await page.getByRole('checkbox', { name: 'Select all' }).check();
      await page.getByRole('button', { name: 'Create in Firefly III (5)' }).click();
      await page.getByRole('dialog').getByRole('button', { name: T.en.confirm }).click();
      await expect.poll(() => applied.length).toBe(1);
      const ms = applied[0].movements;
      expect(byId(ms, 12)).toMatchObject({ counterparty_ff3_id: 51 });
      expect(byId(ms, 15)).toMatchObject({ counterparty_ff3_id: 52 });
      expect(byId(ms, 14)).toMatchObject({ counterparty_name: 'Acme Motors' });
      for (const id of [16, 17]) {
        expect(byId(ms, id)).not.toHaveProperty('counterparty_ff3_id');
        expect(byId(ms, id)).not.toHaveProperty('counterparty_name');
      }
    });

    test('G18 a bulk choice replaces every suggestion', async ({ page }) => {
      const applied = await mockCounterpartyResults(page, [BOOK, SHOES]);
      await page.goto('/results/1');
      await expect(row(page, 'Book').getByText(T.en.hint('Supermarket'))).toBeVisible();
      await page.getByRole('checkbox', { name: 'Select all' }).check();
      await page.getByRole('button', { name: 'Personalize (2)' }).click();
      const bulk = page.getByRole('dialog', { name: 'Personalize 2 movements' });
      await bulk.getByRole('combobox', { name: T.en.dest }).click();
      await page.getByRole('option', { name: /Rainy Day/ }).click();
      await bulk.getByRole('button', { name: 'Apply' }).click();
      await page.getByRole('button', { name: 'Create in Firefly III (2)' }).click();
      await page.getByRole('dialog').getByRole('button', { name: T.en.confirm }).click();
      await expect.poll(() => applied.length).toBe(1);
      for (const m of applied[0].movements) expect(m).toMatchObject({ counterparty_ff3_id: 3 });
    });

    test('G21 the bulk sheet is not pre-filled: no mark, «Leave unchanged», rows keep their own', async ({
      page,
    }) => {
      const applied = await mockCounterpartyResults(page, [BOOK, SHOES]);
      await page.goto('/results/1');
      await page.getByRole('checkbox', { name: 'Select all' }).check();
      await page.getByRole('button', { name: 'Personalize (2)' }).click();
      const bulk = page.getByRole('dialog', { name: 'Personalize 2 movements' });
      const c = bulk.getByRole('combobox', { name: T.en.dest });
      await expect(c).toHaveAttribute('placeholder', 'Leave unchanged');
      await expect(c).toHaveValue('');
      await expect(bulk.getByText(/Suggested/)).toHaveCount(0);
      await expect(bulk.getByRole('button', { name: T.en.clear })).toHaveCount(0);
      // filling another field leaves the account of each row alone
      await bulk.getByRole('textbox', { name: 'Notes' }).fill('n');
      await bulk.getByRole('button', { name: 'Apply' }).click();
      await page.getByRole('button', { name: 'Create in Firefly III (2)' }).click();
      await page.getByRole('dialog').getByRole('button', { name: T.en.confirm }).click();
      await expect.poll(() => applied.length).toBe(1);
      expect(byId(applied[0].movements, 12)).toMatchObject({ counterparty_ff3_id: 51, notes: 'n' });
      expect(byId(applied[0].movements, 15)).toMatchObject({ counterparty_ff3_id: 52, notes: 'n' });
    });

    test('G19 bulk «Leave unchanged» keeps each row suggestion', async ({ page }) => {
      const applied = await mockCounterpartyResults(page, [BOOK, SHOES]);
      await page.goto('/results/1');
      await page.getByRole('checkbox', { name: 'Select all' }).check();
      await page.getByRole('button', { name: 'Personalize (2)' }).click();
      const bulk = page.getByRole('dialog', { name: 'Personalize 2 movements' });
      await bulk.getByRole('combobox', { name: T.en.dest }).click();
      await page.getByRole('option', { name: /Rainy Day/ }).click();
      await bulk.getByRole('combobox', { name: T.en.dest }).click();
      await page.getByRole('option', { name: 'Leave unchanged' }).click();
      await bulk.getByRole('button', { name: 'Apply' }).click();
      await page.getByRole('button', { name: 'Create in Firefly III (2)' }).click();
      await page.getByRole('dialog').getByRole('button', { name: T.en.confirm }).click();
      await expect.poll(() => applied.length).toBe(1);
      expect(byId(applied[0].movements, 12)).toMatchObject({ counterparty_ff3_id: 51 });
      expect(byId(applied[0].movements, 15)).toMatchObject({ counterparty_ff3_id: 52 });
    });
  });

  for (const l of ['es', 'fr'] as const) {
    test(`G20 marks, hint and clear action are translated (${l})`, async ({ page }) => {
      const t = T[l];
      await page.addInitScript((x) => localStorage.setItem('ffm.locale', x), l);
      await mockCounterpartyResults(page, [BOOK, SHOES]);
      await page.goto('/results/1');
      await expect(row(page, 'Book').getByText(t.hint('Supermarket'))).toBeVisible();
      await openSheet(page, 'Book', t);
      await expect(sheet(page, t).getByText(t.learned, { exact: true })).toBeVisible();
      await expect(sheet(page, t).getByRole('button', { name: t.clear })).toBeVisible();
      await page.keyboard.press('Escape');
      await openSheet(page, 'Shoes', t);
      await expect(sheet(page, t).getByText(t.history, { exact: true })).toBeVisible();
    });
  }
});
