import wasm from "../rust/Cargo.toml"
import { App, Modal, Plugin, PluginSettingTab, request, Setting } from 'obsidian';
import { LocalStorageSettings } from "localStorageSettings";

// Remember to rename these classes and interfaces!

interface ObsidianSlackPluginSettings {
	apiToken: string;
	cookie: string;
}

const DEFAULT_SETTINGS: ObsidianSlackPluginSettings = {
	apiToken: 'default',
	cookie: 'default'
}

export default class ObisdianSlackPlugin extends Plugin {
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
	plugin: ObisdianSlackPlugin;
	get_slack_message: (apiToken: string, cookie: string, url: string) => any;

	constructor(app: App, plugin: ObisdianSlackPlugin, get_slack_message: (apiToken: string, cookie: string, url: string) => any) {
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
			console.log(await this.get_slack_message(apiToken, cookie, this.url));
			// 	console.log(await request(
			// 		{
			// 			"url": "https://axon.slack.com/api/conversations.replies?channel=C01ENB4KP26&ts=1671055784.980429&pretty=1&inclusive=true",
			// 			"headers": {
			// 				"content-type": "application/x-www-form-urlencoded",
			// 				"cookie": "d=xoxd-NImkt4e5%2FBZJ8cm8bPd9JWAZ5ATSvnUwE%2FHGRV4E%2FyCdFSbaclP0Xw6p0MwCij7dVH0sG9oLVrO8uVW9DOUP2AmGituX8NwJgd8iVSOnjWCqR%2F%2Fx0KraMm%2FYuBZCJWfVDKxA8df9Yz6OX5XB2qPXA0c9F1DvLbYDZP7btXloR8RdQoEIb5dBdQ%3D%3D;"
			// 			},
			// 			"body": "token=xoxc-4684147883-3183999236788-4411640857313-b8215c23899763f5f3e048dedb3d8e2cdee8957a7f2eaafa7d81eccda9ca35d7",
			// 			"method": "POST"
			// 		}
			// 	))
			// }
			contentEl.empty();
		}
	}
}

class ObsidianSlackPluginSettingsTab extends PluginSettingTab {
	plugin: ObisdianSlackPlugin;

	constructor(app: App, plugin: ObisdianSlackPlugin) {
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
