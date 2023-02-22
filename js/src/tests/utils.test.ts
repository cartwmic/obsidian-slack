/**
 * @jest-environment jsdom
 */

import {
  Notice,
  request,
  RequestUrlParam,
  RequestUrlResponse,
  RequestUrlResponsePromise,
  TFile,
  Vault,
} from "obsidian";
import * as utils from "../utils";

const mockNotice = Notice as jest.Mock<Notice>;

Object.defineProperty(navigator, "clipboard", {
  value: {
    writeText: jest.fn(() => {}),
  },
});

describe("process result logic", () => {
  beforeEach(() => {
    jest.resetAllMocks();
  });

  test("sends notice on success result and successful save of result and copies to clipboard", async () => {
    let mock_result = { message_and_thread: {}, file_name: "mock_filename" };
    let mock_vault = new Vault();

    window.alert = jest.fn((msg: string) => {
      console.log("alert: " + msg);
    });
    mockNotice.mockImplementationOnce((msg: string) => {
      console.log("notice: " + msg);
      return new Notice(msg);
    });

    jest.spyOn(mock_vault, "getConfig")
      .mockImplementation(() => {
        return "abc";
      });
    jest.spyOn(mock_vault, "create")
      .mockImplementation(() => {
        return Promise.resolve(new TFile());
      });

    await utils.process_result("cookie", mock_result, mock_vault);

    expect(window.alert).toBeCalledTimes(0);
    expect(navigator.clipboard.writeText).toBeCalledTimes(1);
    expect(navigator.clipboard.writeText).toBeCalledWith(mock_result.file_name);
    expect(mock_vault.getConfig).toBeCalledTimes(1);
    expect(mock_vault.getConfig).toBeCalledWith("attachmentFolderPath");
    expect(mock_vault.create).toBeCalledTimes(1);
    expect(mock_vault.create).toBeCalledWith(
      "abc/mock_filename",
      JSON.stringify(mock_result, undefined, 2),
    );
    // one extra due to mocking of constructor
    expect(mockNotice).toBeCalledTimes(2);
  });

  test("sends notice on success result and successful save of result, save of attachments, and copies to clipboard", async () => {
    let mock_result = { message_and_thread: {}, file_name: "mock_filename", file_links: new Map([["file1", "link"]]) };
    let mock_vault = new Vault();

    window.alert = jest.fn((msg: string) => {
      console.log("alert: " + msg);
    });
    mockNotice.mockImplementationOnce((msg: string) => {
      console.log("notice: " + msg);
      return new Notice(msg);
    });

    jest.spyOn(mock_vault, "getConfig")
      .mockImplementation(() => {
        return "abc";
      });
    jest.spyOn(mock_vault, "create")
      .mockImplementation(() => {
        return Promise.resolve(new TFile());
      });
    jest.spyOn(mock_vault, "createBinary")
      .mockImplementation((path: string) => {
        return Promise.resolve(new TFile());
      });
    jest.spyOn(utils, "requestUrl")
      .mockImplementation((input) => {
        return Promise.resolve({
          arrayBuffer: new ArrayBuffer(10),
        }) as unknown as RequestUrlResponsePromise;
      });

    await utils.process_result("cookie", mock_result, mock_vault);

    expect(window.alert).toBeCalledTimes(0);
    expect(navigator.clipboard.writeText).toBeCalledTimes(1);
    expect(mock_vault.getConfig).toBeCalledTimes(1);
    expect(mock_vault.getConfig).toBeCalledWith("attachmentFolderPath");
    expect(utils.requestUrl).toHaveBeenCalledWith({
      "headers": { "cookie": "d=cookie" },
      "method": "GET",
      "url": "link",
    });
    expect(mock_vault.create).toHaveBeenCalledWith(
      "abc/mock_filename",
      JSON.stringify(mock_result, utils.replacer, 2),
    );
    expect(mock_vault.createBinary).toHaveBeenCalledWith(
      "abc/file1",
      new ArrayBuffer(10),
    );
    // one extra due to mocking of constructor
    expect(mockNotice).toBeCalledTimes(2);
  });

  test("sends notice on success result and successful save of result, save of attachments, and copies to clipboard even if json file already exists", async () => {
    let mock_result = { message_and_thread: {}, file_name: "mock_filename" };
    let mock_vault = new Vault();

    window.alert = jest.fn((msg: string) => {
      console.log("alert: " + msg);
    });
    mockNotice.mockImplementationOnce((msg: string) => {
      console.log("notice: " + msg);
      return new Notice(msg);
    });

    jest.spyOn(mock_vault, "getConfig")
      .mockImplementation(() => {
        return "abc";
      });
    jest.spyOn(mock_vault, "create")
      .mockImplementationOnce(() => {
        throw new Error("File already exists");
      })
      .mockImplementationOnce(() => {
        return Promise.resolve(new TFile());
      });
    jest.spyOn(mock_vault, "getFiles")
      .mockImplementation(() => {
        let tfile = new TFile();
        tfile.path = mock_result.file_name;
        return [tfile];
      });

    await utils.process_result("cookie", mock_result, mock_vault);

    expect(window.alert).toBeCalledTimes(0);
    expect(navigator.clipboard.writeText).toBeCalledTimes(1);
    expect(mock_vault.getConfig).toBeCalledTimes(1);
    expect(mock_vault.getConfig).toBeCalledWith("attachmentFolderPath");
    expect(mock_vault.getFiles).toBeCalledTimes(1);
    expect(mock_vault.trash).toBeCalledTimes(1);
    expect(mock_vault.create).toBeCalledTimes(2);
    expect(mock_vault.create).toHaveBeenNthCalledWith(
      1,
      "abc/mock_filename",
      JSON.stringify(mock_result, utils.replacer, 2),
    );
    expect(mock_vault.create).toHaveBeenNthCalledWith(
      2,
      "abc/mock_filename",
      JSON.stringify(mock_result, utils.replacer, 2),
    );
    // one extra due to mocking of constructor
    expect(mockNotice).toBeCalledTimes(2);
  });

  test("sends notice on success result and successful save of result, save of attachments, and copies to clipboard even if attachments files already exists", async () => {
    let mock_result = { message_and_thread: {}, file_name: "mock_filename", file_links: new Map([["file1", "link"]]) };
    let mock_vault = new Vault();

    window.alert = jest.fn((msg: string) => {
      console.log("alert: " + msg);
    });
    mockNotice.mockImplementationOnce((msg: string) => {
      console.log("notice: " + msg);
      return new Notice(msg);
    });

    jest.spyOn(mock_vault, "getConfig")
      .mockImplementation(() => {
        return "abc";
      });
    jest.spyOn(mock_vault, "create")
      .mockImplementation(() => {
        return Promise.resolve(new TFile());
      });
    jest.spyOn(mock_vault, "createBinary")
      .mockImplementationOnce((path: string) => {
        throw new Error("File already exists");
      })
      .mockImplementationOnce((path: string) => {
        return Promise.resolve(new TFile());
      });
    jest.spyOn(utils, "requestUrl")
      .mockImplementation((input) => {
        return Promise.resolve({
          arrayBuffer: new ArrayBuffer(10),
        }) as unknown as RequestUrlResponsePromise;
      });
    jest.spyOn(mock_vault, "getFiles")
      .mockImplementation(() => {
        let tfile = new TFile();
        tfile.path = "file1";
        return [tfile];
      });

    await utils.process_result("cookie", mock_result, mock_vault);

    expect(window.alert).toBeCalledTimes(0);
    expect(navigator.clipboard.writeText).toBeCalledTimes(1);
    expect(mock_vault.getConfig).toBeCalledTimes(1);
    expect(mock_vault.getConfig).toBeCalledWith("attachmentFolderPath");
    expect(mock_vault.getFiles).toBeCalledTimes(1);
    expect(mock_vault.trash).toBeCalledTimes(1);
    expect(utils.requestUrl).toHaveBeenCalledWith({
      "headers": { "cookie": "d=cookie" },
      "method": "GET",
      "url": "link",
    });
    expect(mock_vault.create).toHaveBeenCalledWith(
      "abc/mock_filename",
      JSON.stringify(mock_result, utils.replacer, 2),
    );
    expect(mock_vault.createBinary).toHaveBeenNthCalledWith(
      1,
      "abc/file1",
      new ArrayBuffer(10),
    );
    expect(mock_vault.createBinary).toHaveBeenNthCalledWith(
      2,
      "abc/file1",
      new ArrayBuffer(10),
    );
    // one extra due to mocking of constructor
    expect(mockNotice).toBeCalledTimes(2);
  });

  test("sends alert on failed result doesn't save result nor copy to clipboard", async () => {
    let mock_result = "bad_result";
    let mock_vault = new Vault();

    window.alert = jest.fn((msg: string) => {
      console.log("alert: " + msg);
    });

    jest.spyOn(mock_vault, "getConfig");
    jest.spyOn(mock_vault, "create");

    await utils.process_result("cookie", mock_result, mock_vault);

    expect(window.alert).toBeCalledTimes(1);
    expect(navigator.clipboard.writeText).toBeCalledTimes(0);
    expect(mock_vault.getConfig).toBeCalledTimes(0);
    expect(mock_vault.create).toBeCalledTimes(0);
    expect(mockNotice).toBeCalledTimes(0);
  });

  test("sends alert on attachments not saving due to exception, doesn't copy to clipboard", async () => {
    let mock_result = { message_and_thread: {}, file_name: "mock_filename", file_links: new Map([["file1", "link1"]]) };
    let mock_vault = new Vault();

    window.alert = jest.fn((msg: string) => {
      console.log("alert: " + msg);
    });
    mockNotice.mockImplementationOnce((msg: string) => {
      console.log("notice: " + msg);
      return new Notice(msg);
    });

    jest.spyOn(mock_vault, "getConfig")
      .mockImplementation(() => {
        return "abc";
      });
    jest.spyOn(mock_vault, "create")
      .mockImplementation((path: string) => {
        return Promise.resolve(new TFile());
      });
    jest.spyOn(mock_vault, "createBinary")
      .mockImplementation((path: string) => {
        throw new Error("whoops");
      });

    jest.spyOn(utils, "requestUrl")
      .mockImplementation((input) => {
        return Promise.resolve({
          arrayBuffer: new ArrayBuffer(10),
        }) as unknown as RequestUrlResponsePromise;
      });

    await utils.process_result("cookie", mock_result, mock_vault);

    expect(window.alert).toBeCalledTimes(1);
    expect(navigator.clipboard.writeText).toBeCalledTimes(0);
    expect(mock_vault.getConfig).toBeCalledTimes(1);
    expect(mock_vault.getConfig).toBeCalledWith("attachmentFolderPath");
    expect(utils.requestUrl).toHaveBeenCalledWith({
      "headers": { "cookie": "d=cookie" },
      "method": "GET",
      "url": "link1",
    });
    expect(mock_vault.create).toHaveBeenCalledWith(
      "abc/mock_filename",
      JSON.stringify(mock_result, utils.replacer, 2),
    );
    expect(mock_vault.createBinary).toHaveBeenCalledWith(
      "abc/file1",
      new ArrayBuffer(10),
    );
  });

  test("sends alert on file not saving due to exception, doesn't copy to clipboard", async () => {
    let mock_result = { message_and_thread: {}, file_name: "mock_filename" };
    let mock_vault = new Vault();

    window.alert = jest.fn((msg: string) => {
      console.log("alert: " + msg);
    });
    mockNotice.mockImplementationOnce((msg: string) => {
      console.log("notice: " + msg);
      return new Notice(msg);
    });

    jest.spyOn(mock_vault, "getConfig")
      .mockImplementation(() => {
        return "abc";
      });
    jest.spyOn(mock_vault, "create")
      .mockImplementation(() => {
        throw new Error();
      });

    await utils.process_result("cookie", mock_result, mock_vault);

    expect(window.alert).toBeCalledTimes(1);
    expect(navigator.clipboard.writeText).toBeCalledTimes(0);
    expect(mock_vault.getConfig).toBeCalledTimes(1);
    expect(mock_vault.getConfig).toBeCalledWith("attachmentFolderPath");
    expect(mock_vault.create).toBeCalledTimes(1);
    expect(mock_vault.create).toBeCalledWith(
      "abc/mock_filename",
      JSON.stringify(mock_result, undefined, 2),
    );
    expect(mockNotice).toBeCalledTimes(0);
  });

  test("sends alert on file not saving, doesn't copy to clipboard", async () => {
    let mock_result = { message_and_thread: { file_name: "mock_filename" } };
    let mock_vault = new Vault();

    window.alert = jest.fn((msg: string) => {
      console.log("alert: " + msg);
    });
    mockNotice.mockImplementationOnce((msg: string) => {
      console.log("notice: " + msg);
      return new Notice(msg);
    });

    jest.spyOn(mock_vault, "getConfig")
      .mockImplementation(() => {
        return "abc";
      });
    jest.spyOn(mock_vault, "create")
      .mockImplementation(() => {
        throw new Error();
      });
    jest.spyOn(utils, "save_result")
      .mockImplementationOnce(() => {
        return Promise.resolve(false);
      });

    await utils.process_result("cookie", mock_result, mock_vault);

    expect(window.alert).toBeCalledTimes(1);
    expect(navigator.clipboard.writeText).toBeCalledTimes(0);
    expect(utils.save_result).toBeCalledTimes(1);
    expect(utils.save_result).toBeCalledWith("cookie", mock_result, mock_vault);
    expect(mockNotice).toBeCalledTimes(0);
  });
});

describe("get slack message logic", () => {
  beforeEach(() => {
    jest.resetAllMocks();
  });

  test("successfully validates inputs and processes result", async () => {
    let api_token = "an api token";
    let cookie = "a cookie";
    let url = "a url";
    let get_slack_message_func = (
      apiToken: string,
      cookie: string,
      url: string,
      feature_flags: any,
      request_func: (params: RequestUrlParam) => Promise<string>,
    ): Promise<string> => {
      return Promise.resolve("resolve");
    };
    let settings = utils.DEFAULT_SETTINGS;
    let mock_vault = new Vault();

    window.alert = jest.fn((msg: string) => {
      console.log("alert2: " + msg);
    });
    jest.spyOn(utils, "process_result").mockImplementationOnce(() => {
      return Promise.resolve();
    });

    await utils.get_slack_message_modal_on_close_helper(
      api_token,
      cookie,
      url,
      get_slack_message_func,
      settings,
      mock_vault,
    );

    expect(window.alert).toBeCalledTimes(0);
    expect(utils.process_result).toBeCalledTimes(1);
    expect(utils.process_result).toBeCalledWith(cookie, "resolve", mock_vault);
  });

  test("empty api token shows alert and doesn't attempt to process result", async () => {
    let api_token = "";
    let cookie = "a cookie";
    let url = "a url";
    let get_slack_message_func = (
      apiToken: string,
      cookie: string,
      url: string,
      feature_flags: any,
      request_func: (params: RequestUrlParam) => Promise<string>,
    ): Promise<string> => {
      return Promise.resolve("resolve");
    };
    let settings = utils.DEFAULT_SETTINGS;
    let mock_vault = new Vault();

    window.alert = jest.fn((msg: string) => {
      console.log("alert: " + msg);
    });
    jest.spyOn(utils, "process_result");

    await utils.get_slack_message_modal_on_close_helper(
      api_token,
      cookie,
      url,
      get_slack_message_func,
      settings,
      mock_vault,
    );

    expect(window.alert).toBeCalledTimes(1);
    expect(utils.process_result).toBeCalledTimes(0);
  });

  test("empty cookie shows alert and doesn't attempt to process result", async () => {
    let api_token = "a token";
    let cookie = "";
    let url = "a url";
    let get_slack_message_func = (
      apiToken: string,
      cookie: string,
      url: string,
      feature_flags: any,
      request_func: (params: RequestUrlParam) => Promise<string>,
    ): Promise<string> => {
      return Promise.resolve("resolve");
    };
    let settings = utils.DEFAULT_SETTINGS;
    let mock_vault = new Vault();

    window.alert = jest.fn((msg: string) => {
      console.log("alert: " + msg);
    });
    jest.spyOn(utils, "process_result");

    await utils.get_slack_message_modal_on_close_helper(
      api_token,
      cookie,
      url,
      get_slack_message_func,
      settings,
      mock_vault,
    );

    expect(window.alert).toBeCalledTimes(1);
    expect(utils.process_result).toBeCalledTimes(0);
  });

  test("do nothing on empty url", async () => {
    let api_token = "a token";
    let cookie = "a cookie";
    let url = "";
    let get_slack_message_func = (
      apiToken: string,
      cookie: string,
      url: string,
      feature_flags: any,
      request_func: (params: RequestUrlParam) => Promise<string>,
    ): Promise<string> => {
      return Promise.resolve("resolve");
    };
    let settings = utils.DEFAULT_SETTINGS;
    let mock_vault = new Vault();

    window.alert = jest.fn((msg: string) => {
      console.log("alert: " + msg);
    });
    jest.spyOn(utils, "process_result");

    await utils.get_slack_message_modal_on_close_helper(
      api_token,
      cookie,
      url,
      get_slack_message_func,
      settings,
      mock_vault,
    );

    expect(window.alert).toBeCalledTimes(0);
    expect(utils.process_result).toBeCalledTimes(0);
  });
});
