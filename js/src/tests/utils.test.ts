/**
 * @jest-environment jsdom
 */

import { process_result } from "../utils";
import { Vault, TFile, Notice } from "obsidian";

const mockNotice = Notice as jest.Mock<Notice>;

Object.defineProperty(navigator, "clipboard", {
    value: {
        writeText: jest.fn(() => { }),
    },
});

window.alert = jest.fn(() => { });



describe("process result logic", () => {
    test("sends notice on success result and successful save of result", async () => {
        let mock_result = { message_and_thread: { file_name: "mock_filename" } };
        let mock_vault = new Vault();

        mockNotice.mockImplementationOnce((msg: string) => { console.log(msg); return new Notice(msg) });

        jest.spyOn(mock_vault, "getConfig")
            .mockImplementation(() => { return "abc" })
        jest.spyOn(mock_vault, "create")
            .mockImplementation(() => { return Promise.resolve(new TFile()) })

        await process_result(mock_result, mock_vault);

        expect(window.alert).toBeCalledTimes(0);
        expect(navigator.clipboard.writeText).toBeCalledTimes(1);
        expect(mock_vault.getConfig).toBeCalledTimes(1);
        expect(mock_vault.getConfig).toBeCalledWith("attachmentFolderPath");
        expect(mock_vault.create).toBeCalledTimes(1);
        expect(mock_vault.create).toBeCalledWith("abc/mock_filename", JSON.stringify(mock_result.message_and_thread, undefined, 2));
        // one extra due to mocking of constructor
        expect(mockNotice).toBeCalledTimes(2);
    })
})