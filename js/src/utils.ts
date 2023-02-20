import { Notice, request, RequestUrlParam, Vault } from "obsidian";
import * as path from "path";
// cyclic dependency to work correctly with jest unit testing. See https://stackoverflow.com/a/47976589
import * as mod from "./utils";

export interface ObsidianSlackPluginSettings {
  apiToken: string;
  cookie: string;
  get_users: boolean;
  get_channel_info: boolean;
  get_file_data: boolean;
  get_team_info: boolean;
}

export const DEFAULT_SETTINGS: ObsidianSlackPluginSettings = {
  apiToken: "default",
  cookie: "default",
  get_users: false,
  get_channel_info: false,
  get_file_data: false,
  get_team_info: false,
};

export async function process_result(result: any, vault: Vault) {
  try {
    if (typeof (result) === "string") {
      alert(result);
      return;
    }

    let file_saved = await mod.save_result(result, vault);

    if (file_saved) {
      await navigator.clipboard.writeText(result.message_and_thread.file_name);
      let message =
        "Successfully downloaded slack message and saved to attachment folder. File name saved to clipboard";
      new Notice(message);
    } else {
      alert("File saving was unsuccessful");
    }
  } catch (e) {
    let message = "There was a problem saving message results: " + e;
    console.log(message);
    alert(message);
  }
}

export async function save_result(result: any, vault: Vault): Promise<boolean> {
  let attachment_path = vault.getConfig("attachmentFolderPath");
  let file_path = path.join(attachment_path, result.file_name);
  let keys_to_save = ["message_and_thread", "users", "channel", "teams", "file_name"];
  let result_filtered = Object.keys(result).filter((key) => keys_to_save.includes(key)).reduce((obj, key) => {
    return Object.assign(obj, {
      [key]: result[key],
    });
  }, {});
  let tfile = await vault.create(file_path, JSON.stringify(result_filtered, undefined, 2));
  console.log(tfile);
  if (result.file_data) {
    console.log(result.file_data);
    for (let key in result.file_data) {
      await vault.create(path.join(attachment_path, key), result.file_data[key]);
    }
  }
  return tfile ? true : false;
}

export async function get_slack_message_modal_on_close_helper(
  api_token: string | null,
  cookie: string | null,
  url: string,
  get_slack_message_func: (
    apiToken: string,
    cookie: string,
    url: string,
    feature_flags: any,
    request_func: (params: RequestUrlParam) => Promise<string>,
  ) => any,
  settings: ObsidianSlackPluginSettings,
  vault: Vault,
): Promise<any> {
  if (api_token && cookie) {
    // do nothing on empty url
    if (url) {
      let result = await get_slack_message_func(api_token, cookie, url, {
        "get_users": settings.get_users,
        "get_channel_info": settings.get_channel_info,
        "get_file_data": settings.get_file_data,
        "get_team_info": settings.get_team_info,
      }, request);

      await mod.process_result(result, vault);
    }
  } else {
    alert("apiToken or cookie or url was null, undefined, or empty. Aborting operation");
  }
}
