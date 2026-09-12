import type { Page, Locator } from '@playwright/test';

export class ImportWizardPage {
  readonly page: Page;
  readonly dialog: Locator;
  readonly accountSelect: Locator;
  readonly dropzone: Locator;
  readonly fileInput: Locator;
  readonly previewTable: Locator;
  readonly invalidSection: Locator;
  readonly mappingDateSelect: Locator;
  readonly mappingAmountSelect: Locator;
  readonly mappingDescSelect: Locator;
  readonly mappingOriginSelect: Locator;
  readonly reconcileAccountSelect: Locator;
  readonly toleranceButtons: Locator;
  readonly reconcileButton: Locator;
  readonly cancelButton: Locator;
  readonly backButton: Locator;
  readonly nextButton: Locator;
  readonly stepIndicator: Locator;

  constructor(page: Page) {
    this.page = page;
    this.dialog = page.getByRole('dialog');
    this.accountSelect = page.getByRole('combobox', { name: /Firefly III account|Cuenta de Firefly/ }).first();
    this.dropzone = page.getByRole('button', { name: /Drag CSV|Arrastra/ }).or(page.locator('[role="button"]').filter({ hasText: /Drop to upload|Suelta para subir/ }).first());
    // Hidden file input
    this.fileInput = page.locator('input[type="file"]');
    this.previewTable = page.getByRole('table');
    this.invalidSection = page.getByText(/Invalid rows|Filas inválidas/);
    this.mappingDateSelect = page.getByRole('combobox', { name: /Date column|Columna de fecha/ });
    this.mappingAmountSelect = page.getByRole('combobox', { name: /Amount column|Columna de importe/ });
    this.mappingDescSelect = page.getByRole('combobox', { name: /Description column|Columna de descripción/ });
    this.mappingOriginSelect = page.getByRole('combobox', { name: /Origin column|Columna de origen/ });
    this.reconcileAccountSelect = page.getByRole('combobox', { name: /Firefly III account|Cuenta de Firefly/ }).last();
    this.toleranceButtons = page.getByRole('button', { name: /^0$|±1|±2|±3/ });
    this.reconcileButton = page.getByRole('button', { name: /Reconcile|Conciliar/ });
    this.cancelButton = page.getByRole('button', { name: /Cancel|Cancelar/ });
    this.backButton = page.getByRole('button', { name: /Back|Atrás/ });
    this.nextButton = page.getByRole('button', { name: /Next|Siguiente/ });
    this.stepIndicator = page.locator('[data-testid="wizard-steps"], .wizard-steps, [role="progressbar"]');
  }

  async gotoImports() {
    await this.page.goto('/imports');
  }

  async openWizard() {
    await this.page.getByRole('button', { name: /New import|Nueva importación/ }).click();
  }

  async openWizardResume(batchId: number) {
    await this.page.goto('/imports');
    const row = this.page.locator('tbody tr', { hasText: String(batchId) });
    // Find the resume button (Play) for that row
    await row.getByRole('button', { name: /Resume|Reanudar/ }).click();
  }

  async selectAccount(name: string) {
    await this.accountSelect.click();
    await this.page.getByRole('option', { name }).click();
  }

  async uploadFile(path: string) {
    // Use file chooser for hidden input
    const fileChooserPromise = this.page.waitForEvent('filechooser');
    await this.dropzone.click();
    const fileChooser = await fileChooserPromise;
    await fileChooser.setFiles(path);
  }

  async selectMapping(columns: { date: string; amount: string; desc: string; origin?: string }) {
    await this.mappingDateSelect.click();
    await this.page.getByRole('option', { name: columns.date }).click();
    await this.mappingAmountSelect.click();
    await this.page.getByRole('option', { name: columns.amount }).click();
    await this.mappingDescSelect.click();
    await this.page.getByRole('option', { name: columns.desc }).click();
    if (columns.origin) {
      await this.mappingOriginSelect.click();
      await this.page.getByRole('option', { name: columns.origin }).click();
    }
  }

  async expectStep(n: number) {
    // WizardShell shows step dots, check current step indicator
    await this.page.getByText(new RegExp(`Step ${n} of 4|Paso ${n} de 4`)).waitFor({ state: 'visible', timeout: 3000 });
  }
}
