import type { Page, Locator } from '@playwright/test';

export class ImportsPage {
  readonly page: Page;
  readonly heading: Locator;
  readonly searchInput: Locator;
  readonly statusSelect: Locator;
  readonly fromInput: Locator;
  readonly toInput: Locator;
  readonly table: Locator;
  readonly rows: Locator;
  readonly emptyState: Locator;
  readonly skeleton: Locator;
  readonly refreshButton: Locator;
  readonly newImportButton: Locator;
  readonly deleteDialog: Locator;
  readonly deleteDialogTitle: Locator;
  readonly deleteConfirmButton: Locator;
  readonly deleteCancelButton: Locator;

  constructor(page: Page) {
    this.page = page;
    this.heading = page.getByRole('heading', { name: /Imports|Importaciones/ });
    this.searchInput = page.getByPlaceholder(/Search by filename|Buscar por nombre/);
    this.statusSelect = page.getByRole('combobox', { name: /Filter by status|Filtrar por estado/ });
    this.fromInput = page.getByRole('textbox', { name: /^From|Desde$/ });
    // To input appears twice (filter + dialog), use aria-label
    this.toInput = page.getByRole('textbox', { name: /^To|Hasta$/ });
    this.table = page.getByRole('table');
    this.rows = page.locator('tbody tr');
    this.emptyState = page.getByText(/No imports yet|Aún no hay importaciones/);
    this.skeleton = page.locator('[aria-busy="true"]');
    this.refreshButton = page.getByRole('button', { name: '' }).filter({ has: page.locator('.lucide-refresh-cw') }).first();
    // Fallback: button with RefreshCw icon is the first icon button in header
    this.newImportButton = page.getByRole('button', { name: /New import|Nueva importación/ });
    this.deleteDialog = page.getByRole('dialog');
    this.deleteDialogTitle = page.getByRole('heading', { name: /Delete import|Eliminar importación/ });
    this.deleteConfirmButton = this.deleteDialog.getByRole('button', { name: /Confirm|Confirmar/ });
    this.deleteCancelButton = this.deleteDialog.getByRole('button', { name: /Cancel|Cancelar/ });
  }

  async goto() {
    await this.page.goto('/imports');
  }

  async search(text: string) {
    await this.searchInput.fill(text);
  }

  async filterStatus(status: string) {
    await this.statusSelect.click();
    await this.page.getByRole('option', { name: new RegExp(`^${status}$`, 'i') }).click();
  }

  async setFrom(date: string) {
    await this.fromInput.fill(date);
  }

  async setTo(date: string) {
    await this.toInput.fill(date);
  }

  getRowByFilename(name: string): Locator {
    return this.page.locator('tbody tr', { hasText: name });
  }

  getActionButton(filename: string, action: 'resume' | 'view' | 'rerun' | 'delete'): Locator {
    const row = this.getRowByFilename(filename);
    // Use aria-label which is translated, but icons are inside buttons with title
    const labelMap: Record<string, RegExp> = {
      resume: /Resume|Reanudar/,
      view: /View results|Ver resultados/,
      rerun: /Re-run|Reejecutar/,
      delete: /Delete|Eliminar/,
    };
    return row.getByRole('button', { name: labelMap[action] });
  }
}
