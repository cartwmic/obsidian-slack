import wasm from "../rust/Cargo.toml"
import { App, Modal, Notice, Plugin, PluginSettingTab, Setting, Vault } from 'obsidian';
import { LocalStorageSettings } from "localStorageSettings";


interface ObsidianSlackPluginSettings {
	apiToken: string;
	cookie: string;
}

const DEFAULT_SETTINGS: ObsidianSlackPluginSettings = {
	apiToken: 'default',
	cookie: 'default'
}

export default class ObsidianSlackPlugin extends Plugin {
	settings: ObsidianSlackPluginSettings;
	localStorage: LocalStorageSettings;

	async onload() {
		this.localStorage = new LocalStorageSettings(this);
		await this.loadSettings();
		const exports = await wasm();
		exports.init_wasm(undefined);

		this.addCommand({
			id: 'get-slack-message',
			name: 'Get Slack Message by URL',
			callback: () => {
				new GetSlackMessageModal(this.app, this, exports.get_slack_message).open();
			}
		});

		this.addSettingTab(new ObsidianSlackPluginSettingsTab(this.app, this));
	}

	async loadSettings() {
		this.settings = Object.assign({}, DEFAULT_SETTINGS, await this.loadData());
	}

	async saveSettings() {
		await this.saveData(this.settings);
	}
}

class GetSlackMessageModal extends Modal {
	url: string;
	plugin: ObsidianSlackPlugin;
	get_slack_message: (apiToken: string, cookie: string, url: string, vault: Vault) => any;

	constructor(app: App, plugin: ObsidianSlackPlugin, get_slack_message: (apiToken: string, cookie: string, url: string, vault: Vault) => any) {
		super(app);
		this.get_slack_message = get_slack_message;
		this.plugin = plugin;
	}

	onOpen() {
		const { contentEl, titleEl } = this;
		titleEl.setText('Get Slack Message by URL');
		contentEl.setText('Paste URL below and submit')
		const div = contentEl.createDiv();

		const text = div
			.createEl("input", {
			});

		div.createEl("button",
			{
				text: "Submit",
			})
			.addEventListener("click", async () => {
				console.log(text.value);
				this.url = text.value;
				this.close();
			});
	}

	async onClose() {
		const { contentEl } = this;
		var apiToken = this.plugin.localStorage.getApiToken();
		var cookie = this.plugin.localStorage.getCookie();
		if (apiToken === null || cookie === null) {
			alert("apiToken or cookie was null, aborting operation")
		}
		else {
			await this.get_slack_message(apiToken, cookie, this.url, this.app.vault);
		}
		contentEl.empty();
	}
}

class ObsidianSlackPluginSettingsTab extends PluginSettingTab {
	plugin: ObsidianSlackPlugin;

	constructor(app: App, plugin: ObsidianSlackPlugin) {
		super(app, plugin);
		this.plugin = plugin;
	}

	display(): void {
		const { containerEl } = this;

		containerEl.empty();

		containerEl.createEl('h2', { text: 'Settings for obsidian slack.' });

		new Setting(containerEl)
			.setName('API Token')
			.setDesc('Token used to authenticate requests to the Slack API, you won\'t be able to see it again.')
			.addText(text => text
				.setPlaceholder('Enter your token')
				.onChange(async (value) => {
					console.log('onChange:token: ' + value);
					this.plugin.localStorage.setApiToken(value);
				}));

		new Setting(containerEl)
			.setName('Cookie')
			.setDesc('Cookie used to authenticate requests to the Slack API, you won\'t be able to see it again.')
			.addText(text => text
				.setPlaceholder('Enter your cookie')
				.onChange(async (value) => {
					console.log('onChange:cookie: ' + value);
					this.plugin.localStorage.setCookie(value);
				}));
	}
}

export function combine_result(timestamp_result: any, thread_timestamp_result: any): any {
	try {
		return {
			"timestamp": timestamp_result,
			"thread_timestamp": thread_timestamp_result
		}
	} catch (error) {
		console.log(timestamp_result);
		console.log(thread_timestamp_result)
	}
}
