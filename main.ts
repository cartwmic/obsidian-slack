import { App, Plugin, PluginSettingTab, Setting } from 'obsidian';

interface ObsidianSlackSettings {
	slackAPItoken: string;
}

export default class ObsidianSlack extends Plugin {
	settings: ObsidianSlackSettings;

	async onload() {
		await this.loadSettings();

		// This adds a simple command that can be triggered anywhere
		this.addCommand({
			id: 'download-slack-messages-across-vault',
			name: 'Download slack messages across vault',
			callback: () => {
				// insert rust function here to download slack messages and save them to attachments location. Make sure we don't redownload already existing messages
				console.log("hello")
				// pass token
				// pass file list
				// pass save location
			}
		})

		this.addCommand({
			id: 'download-slack-messages-in-current-file',
			name: 'Download slack messages in current file',
			callback: () => {
				// insert rust function here to download slack messages and save them to attachments location but only for current file. Make sure we don't redownload already existing messages
				console.log("hello again")
				// pass token
				// pass file list
				// pass save location
			}
		})

		this.addSettingTab(new SettingsTab(this.app, this));
	}

	onunload() {

	}

	async loadSettings() {
		this.settings = Object.assign({}, await this.loadData());
	}

	async saveSettings() {
		await this.saveData(this.settings);
	}
}


class SettingsTab extends PluginSettingTab {
	plugin: ObsidianSlack;

	constructor(app: App, plugin: ObsidianSlack) {
		super(app, plugin);
		this.plugin = plugin;
	}

	display(): void {
		const {containerEl} = this;

		containerEl.empty();

		containerEl.createEl('h2', {text: 'Settings for Obsidian Slack'});

		new Setting(containerEl)
			.setName('User API Token')
			.setDesc('API token used to communicate with Slack API')
			.addText(text => text
				.setPlaceholder('Enter your token')
				.setValue(this.plugin.settings.slackAPItoken)
				.onChange(async (value) => {
					this.plugin.settings.slackAPItoken = value;
					await this.plugin.saveSettings();
				}));
	}
}
