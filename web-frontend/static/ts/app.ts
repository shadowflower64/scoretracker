import { select, selectAny } from "./Component.js";
import { SongTablePage } from "./page/SongTablePage.js";
import { makeTask } from "./tasks/Tasks.js";
import { sleep } from "./Util.js";
import { UUID4 } from "./uuid.js";
console.log("a", UUID4.generate());

console.log("Hello from typescript!");


// const fetched1 = await fetchText("/hey");
// const fetched2 = await sendData("/echo", "POST", "test echo");
const fetched1 = "fetched1";
const fetched2 = "fetched2";

export const screenDiv = select(document, "div", "#screen");
const testEl = selectAny(document, "#test");
testEl.textContent = `Hello from typescript! req1: ${fetched1} req2: ${fetched2}`;

const page = new SongTablePage("in_falsus");
page.open();


const makeTaskBtn = select(document, "button", "#make-task-test");
makeTaskBtn.addEventListener("click", async () => {
    const result = await makeTask("Test task", undefined, async (updateStatus) => {
        updateStatus({ maxProgress: 30 });

        for (let i = 0; i < 30; i++) {
            await sleep(100);
            updateStatus({ progress: i, status: "catching some 'z's" });
        }

        if (confirm("return success?")) {
            return "123asd";
        } else {
            throw new Error("456fgh");
        }
    });
    console.log(result);
});