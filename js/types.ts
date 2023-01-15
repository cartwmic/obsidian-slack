declare module "obsidian" {
  interface App {
    loadLocalStorage(key: string): string | null;
    saveLocalStorage(key: string, value: string | undefined): void;
  }

  interface Vault {
    getConfig(key: string): string;
  }
}

export {};
