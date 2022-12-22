import ObsidianSlackPlugin from "./index";
export class LocalStorageSettings {
    private prefix: string;
    constructor(private readonly plugin: ObsidianSlackPlugin) {
        this.prefix = this.plugin.manifest.id + ":";
    }

    getApiToken(): string | null {
        return app.loadLocalStorage(this.prefix + "apiToken");
    }

    setApiToken(value: string): void {
        return app.saveLocalStorage(this.prefix + "apiToken", value);
    }
}