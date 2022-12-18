import wasm from "../rust/Cargo.toml"
import { App, Plugin, PluginSettingTab, Setting } from 'obsidian';

// Remember to rename these classes and interfaces!

interface ObsidianSlackPluginSettings {
	apiToken: string;
}

const DEFAULT_SETTINGS: ObsidianSlackPluginSettings = {
	apiToken: 'default'
}

export default class ObisdianSlackPlugin extends Plugin {
	settings: ObsidianSlackPluginSettings;

	async onload() {
		await this.loadSettings();
		const exports = await wasm();

		// This creates an icon in the left ribbon.
		this.addRibbonIcon('dice', 'Sample Plugin', (evt: MouseEvent) => {
			// Called when the user clicks the icon.
            exports.greet();
		});

        // This adds a settings tab so the user can configure various aspects of the plugin
		this.addSettingTab(new ObsidianSlackPluginSettingsTab(this.app, this));
	}

	async loadSettings() {
		this.settings = Object.assign({}, DEFAULT_SETTINGS, await this.loadData());
	}

	async saveSettings() {
		await this.saveData(this.settings);
	}
}

class ObsidianSlackPluginSettingsTab extends PluginSettingTab {
	plugin: ObisdianSlackPlugin;

	constructor(app: App, plugin: ObisdianSlackPlugin) {
		super(app, plugin);
		this.plugin = plugin;
	}

	display(): void {
		const {containerEl} = this;

		containerEl.empty();

		containerEl.createEl('h2', {text: 'Settings for obsidian slack.'});

		new Setting(containerEl)
			.setName('API Token')
			.setDesc('Token used to authenticate requests to the Slack API')
			.addText(text => text
				.setPlaceholder('Enter your secret')
				.setValue(this.plugin.settings.apiToken)
				.onChange(async (value) => {
					console.log('Secret: ' + value);
					this.plugin.settings.apiToken = value;
					await this.plugin.saveSettings();
				}));
	}
}
