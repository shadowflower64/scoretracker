import { select, selectAny } from "./Component.js";
import { SongTablePage } from "./page/SongTablePage.js";
import { makeTask } from "./tasks/Tasks.js";
import { fetchText, sendRequest, sleep } from "./Util.js";
import { UUID4 } from "./lib/uuid.js";
console.log("a", UUID4.generate());

console.log("Hello from typescript!");

export const screenDiv = select(document, "div", "#screen");


const testEl1 = selectAny(document, "#test1");
const testEl2 = selectAny(document, "#test2");
const testEl3 = selectAny(document, "#test3");
testEl1.textContent = `typescript: Hello from typescript!`;
fetchText("/hey").then((value) => testEl2.textContent = `fetch: ${value}`);
sendRequest("/echo", "POST", "test echo").then((value) => testEl3.textContent = `post request: ${value}`);

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
});;;
