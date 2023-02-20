import { App, Modal, Notice, Plugin, PluginSettingTab, request, RequestUrlParam, Setting, Vault } from "obsidian";
import { LocalStorageSettings } from "src/localStorageSettings";
import wasm from "../../rust/Cargo.toml";
import { DEFAULT_SETTINGS, get_slack_message_modal_on_close_helper, ObsidianSlackPluginSettings } from "./utils";

export default class ObsidianSlackPlugin extends Plugin {
  settings: ObsidianSlackPluginSettings;
  localStorage: LocalStorageSettings;

  async onload() {
    this.localStorage = new LocalStorageSettings(this);
    await this.loadSettings();
    const exports = await wasm();
    exports.init_wasm(undefined);

    this.addCommand({
      id: "get-slack-message",
      name: "Get Slack Message by URL",
      callback: () => {
        new GetSlackMessageModal(this.app, this, exports.get_slack_message).open();
      },
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

export class GetSlackMessageModal extends Modal {
  url: string;
  plugin: ObsidianSlackPlugin;
  get_slack_message: (
    apiToken: string,
    cookie: string,
    url: string,
    feature_flags: any,
    request_func: (params: RequestUrlParam) => Promise<string>,
  ) => any;

  constructor(
    app: App,
    plugin: ObsidianSlackPlugin,
    get_slack_message: (
      apiToken: string,
      cookie: string,
      url: string,
      feature_flags: any,
      request_func: (params: RequestUrlParam) => Promise<string>,
    ) => any,
  ) {
    super(app);
    this.get_slack_message = get_slack_message;
    this.plugin = plugin;
  }

  onOpen() {
    const { contentEl, titleEl } = this;
    titleEl.setText("Get Slack Message by URL");
    contentEl.setText("Paste URL below and submit");

    const div0 = contentEl.createDiv();
    div0.createEl("br");

    const div1 = contentEl.createDiv();

    const text = div1
      .createEl("input", {});

    const div2 = contentEl.createDiv();

    div2.createEl("button", {
      cls: ["mod-cta", "obsidian-git-center-button"],
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
    await get_slack_message_modal_on_close_helper(
      apiToken,
      cookie,
      this.url,
      this.get_slack_message,
      this.plugin.settings,
      this.app.vault,
    );
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

    containerEl.createEl("h2", { text: "Settings for obsidian slack." });

    new Setting(containerEl)
      .setName("API Token")
      .setDesc("Token used to authenticate requests to the Slack API, you won't be able to see it again.")
      .addText(text =>
        text
          .setPlaceholder("Enter your token")
          .onChange(async (value) => {
            console.log("onChange:token: " + value);
            this.plugin.localStorage.setApiToken(value);
          })
      );

    new Setting(containerEl)
      .setName("Cookie")
      .setDesc("Cookie used to authenticate requests to the Slack API, you won't be able to see it again.")
      .addText(text =>
        text
          .setPlaceholder("Enter your cookie")
          .onChange(async (value) => {
            this.plugin.localStorage.setCookie(value);
          })
      );

    new Setting(containerEl)
      .setName("Get User Info")
      .setDesc("Toggle if Obsidian Slack should download user info along with messages")
      .addToggle(toggle =>
        toggle
          .setValue(this.plugin.settings.get_users)
          .onChange(async (value) => {
            this.plugin.settings.get_users = value;
            await this.plugin.saveSettings();
          })
      );

    new Setting(containerEl)
      .setName("Get Channel Info")
      .setDesc("Toggle if Obsidian Slack should download channel info along with messages")
      .addToggle(toggle =>
        toggle
          .setValue(this.plugin.settings.get_channel_info)
          .onChange(async (value) => {
            this.plugin.settings.get_channel_info = value;
            await this.plugin.saveSettings();
          })
      );

    new Setting(containerEl)
      .setName("Get Attachments")
      .setDesc("Toggle if Obsidian Slack should download attachments along with messages")
      .addToggle(toggle =>
        toggle
          .setValue(this.plugin.settings.get_file_data)
          .onChange(async (value) => {
            this.plugin.settings.get_file_data = value;
            await this.plugin.saveSettings();
          })
      );

    new Setting(containerEl)
      .setName("Get Team Info")
      .setDesc("Toggle if Obsidian Slack should download team info along with messages")
      .addToggle(toggle =>
        toggle
          .setValue(this.plugin.settings.get_team_info)
          .onChange(async (value) => {
            this.plugin.settings.get_team_info = value;
            await this.plugin.saveSettings();
          })
      );
  }
}
