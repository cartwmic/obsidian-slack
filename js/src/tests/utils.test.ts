/**
 * @jest-environment jsdom
 */

// import { GetSlackMessageModal } from "./index"
// import ObsidianSlackPlugin from "./index";
// import { get_slack_message } from "../rust/target/rollup-plugin-rust/obsidian_slack";
// import { App } from "obsidian";
// jest.mock("../rust/target/rollup-plugin-rust/obsidian_slack")
import { process_result } from "../utils";
import { Vault, TFile } from "obsidian";
// import obsidian from "__mocks__/obsidian";

Object.defineProperty(navigator, "clipboard", {
    value: {
        writeText: jest.fn().mockReturnValueOnce(Promise.resolve(42)),
    },
});


describe("process result logic", () => {
    test("sends notice on success result and successful save of result", async () => {
        let mock_notice_func = jest.fn((message: string) => { });
        let mock_alert_func = jest.fn((message: string) => { })
        let mock_result = { message_and_thread: { file_name: "mock_filename" } };
        let mock_vault = new Vault()

        console.log("here");


        jest.spyOn(mock_vault, "getConfig")
            .mockImplementation(() => { return "abc" })
        jest.spyOn(mock_vault, "create")
            .mockImplementation(() => { return Promise.resolve(new TFile()) })

        await process_result(mock_result, mock_notice_func, mock_vault, mock_alert_func);

        console.log("here2");
        expect(mock_alert_func).toBeCalledTimes(0);
        expect(mock_notice_func).toBeCalledTimes(1);
    })
})