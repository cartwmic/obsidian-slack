import { Notice, request, requestUrl, RequestUrlParam, TFile, Vault } from "obsidian";
import * as path from "path";
// cyclic dependency to work correctly with jest unit testing. See https://stackoverflow.com/a/47976589
import * as mod from "./utils";

export { requestUrl } from "obsidian";

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

export async function process_result(cookie: string, result: any, vault: Vault) {
  try {
    if (typeof (result) === "string") {
      alert(result);
      return;
    }

    let file_saved = await mod.save_result(cookie, result, vault);

    if (file_saved) {
      await navigator.clipboard.writeText(result.file_name);
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

export async function save_result(cookie: string, result: any, vault: Vault): Promise<boolean> {
  console.log(result);

  let attachment_path = vault.getConfig("attachmentFolderPath");
  let file_path = path.join(attachment_path, result.file_name);
  let tfiles: TFile[] = [];
  let result_data = JSON.stringify(result, replacer, 2);
  try {
    tfiles = [await vault.create(file_path, result_data)];
  } catch (e: unknown) {
    if (e instanceof Error) {
      if (e.message.includes("File already exists")) {
        let tfile = vault.getFiles().filter((file) => file_path.includes(file.path))[0];
        await vault.trash(tfile, true);
        tfiles = [await vault.create(file_path, result_data)];
      } else {
        throw e;
      }
    }
  }
  if (result.file_links) {
    for (const [key, val] of result.file_links) {
      let request_url_params: RequestUrlParam = {
        url: val,
        method: "GET",
        headers: {
          cookie: "d=" + cookie,
        },
      };
      let data = await requestUrl(request_url_params);
      console.log(data);
      let file_path = path.join(attachment_path, key);
      try {
        tfiles = tfiles.concat([await vault.createBinary(file_path, data.arrayBuffer)]);
      } catch (e: unknown) {
        if (e instanceof Error) {
          if (e.message.includes("File already exists")) {
            let tfile = vault.getFiles().filter((file) => file_path.includes(file.path))[0];
            await vault.trash(tfile, true);
            tfiles = [await vault.createBinary(file_path, data.arrayBuffer)];
          } else {
            throw e;
          }
        }
      }
    }
  }
  console.log(tfiles);
  return tfiles ? true : false;
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

      await mod.process_result(cookie, result, vault);
    }
  } else {
    alert("apiToken or cookie or url was null, undefined, or empty. Aborting operation");
  }
}

export function replacer(key: any, value: any[]) {
  if (value instanceof Map) {
    return {
      dataType: "Map",
      value: Array.from(value.entries()), // or with spread: value: [...value]
    };
  } else {
    return value;
  }
}
