import type { Page, Locator } from '@playwright/test';

export class SettingsPage {
  readonly page: Page;
  readonly heading: Locator;
  readonly urlInput: Locator;
  readonly tokenInput: Locator;
  readonly tokenKeepHint: Locator;
  readonly testConnectionButton: Locator;
  readonly saveButtons: Locator;
  readonly appearanceHeading: Locator;
  readonly themeLightButton: Locator;
  readonly themeDarkButton: Locator;
  readonly languageHeading: Locator;
  readonly languageSelect: Locator;
  readonly creationDefaultsHeading: Locator;
  readonly tagSelect: Locator;
  readonly originTraceSwitch: Locator;

  constructor(page: Page) {
    this.page = page;
    this.heading = page.getByRole('heading', { name: /Settings|Ajustes|Paramètres/ });
    this.urlInput = page.locator('#ff3-url');
    this.tokenInput = page.locator('#ff3-token');
    this.tokenKeepHint = page.getByText(/Leave empty to keep|Dejar vacío para conservar|Laisser vide pour conserver/);
    this.testConnectionButton = page.getByRole('button', { name: /Test connection|Probar conexión|Tester la connexion/ });
    this.saveButtons = page.getByRole('button', { name: /Save|Guardar|Sauvegarder/ });
    this.appearanceHeading = page.getByRole('heading', { name: /Appearance|Apariencia|Apparence/ });
    this.themeLightButton = page.getByRole('button', { name: /Light|Claro|Clair/ });
    this.themeDarkButton = page.getByRole('button', { name: /Dark|Oscuro|Sombre/ });
    this.languageHeading = page.getByRole('heading', { name: /Language|Idioma|Langue/ });
    this.languageSelect = page.getByRole('combobox', { name: /Language|Idioma|Langue/ });
    this.creationDefaultsHeading = page.getByText(/Creation defaults|Valores por defecto|Valeurs par défaut/);
    this.tagSelect = page.getByRole('combobox', { name: /Default creation tag|Etiqueta de creación|Tag de création/ });
    this.originTraceSwitch = page.getByRole('switch');
  }

  async goto() {
    await this.page.goto('/settings');
  }

  async fillConnection(url: string, token: string) {
    await this.urlInput.fill(url);
    await this.tokenInput.fill(token);
  }

  async expectToast(text: string) {
    await this.page.getByText(text).first().waitFor({ state: 'visible', timeout: 5000 });
  }
}
