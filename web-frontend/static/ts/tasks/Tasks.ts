import { ComponentTemplate, onRemove, place, select } from "../Component.js";

export type UpdateStatusOptions = { status?: string | null, progress?: number, maxProgress?: number; };
export type UpdateStatusFn = (options: UpdateStatusOptions) => void;
export type TaskFunc<T> = (updateStatus: UpdateStatusFn) => Promise<T>;
export type TaskState = { stage: "initial" | "started"; } | { stage: "finished_success"; result: any; } | { stage: "finished_error"; error: any; };
export type TaskInfo = {
    title: string,
    statusText: string | null;
    progress: number;
    maxProgress: number;
    startTime: number,
    lastUpdateTime: number;
    state: TaskState;
};

const taskList: TaskInfo[] = [];
const activeTaskList: TaskInfo[] = [];

export function isTaskActive(task: TaskInfo) {
    return task.state.stage === "started";
}

function getSecondsAgo(earlier: number, now: number = Date.now()) {
    const duration = now - earlier;
    const seconds = Math.floor(duration / 1000);
    if (seconds < 1) {
        return "just now";
    } else {
        return `${seconds} seconds ago`;
    }
}

export const TaskRow = ComponentTemplate.named("task-row", (f, params: { taskInfo: TaskInfo; }) => {
    const title = select(f, "span", "#title");
    const stateText = select(f, "span", "#state-text");
    const statusText = select(f, "span", "#status-text");
    const progressDiv = select(f, "div", "#progress-container");
    const progressNum = select(f, "span", "#progress-num");
    const maxProgressNum = select(f, "span", "#max-progress-num");
    const progress = select(f, "progress", "#progress");
    const lastUpdateSpan = select(f, "span", "#last-update");

    function updateDOM(taskInfo: TaskInfo) {
        title.textContent = taskInfo.title;
        statusText.textContent = taskInfo.statusText;
        progressNum.textContent = taskInfo.progress.toString();
        maxProgressNum.textContent = taskInfo.maxProgress.toString();
        progress.value = taskInfo.progress;
        progress.max = taskInfo.maxProgress;
        lastUpdateSpan.textContent = "just now";
        ;

        if (taskInfo.state.stage === "started") {
            stateText.textContent = "Working...";
        } else if (taskInfo.state.stage === "finished_success") {
            stateText.textContent = "Task finished successfully";
            progressDiv.style.display = "none";
        } else if (taskInfo.state.stage === "finished_error") {
            stateText.textContent = "Task failed";
        }
    }

    const autoUpdate = setInterval(() => {
        lastUpdateSpan.textContent = getSecondsAgo(params.taskInfo.lastUpdateTime);
    }, 1000);
    onRemove(lastUpdateSpan, () => {
        clearInterval(autoUpdate);
    });
    return { updateDOM };
});

export async function makeTask<T>(title: string, maxProgress: number = 0, taskFunc: TaskFunc<T>): Promise<T> {
    const now = Date.now();
    const taskInfo: TaskInfo = {
        title,
        statusText: null,
        progress: 0,
        maxProgress,
        startTime: now,
        lastUpdateTime: now,
        state: { stage: "initial" },
    };
    const taskComponent = TaskRow.create({ taskInfo });
    function updateStatus(options: UpdateStatusOptions) {
        let dirty = false;
        if (options.status) {
            if (taskInfo.statusText !== options.status) dirty = true;
            taskInfo.statusText = options.status;
        }
        if (options.progress) {
            if (taskInfo.progress !== options.progress) dirty = true;
            taskInfo.progress = options.progress;
        }
        if (options.maxProgress) {
            if (taskInfo.maxProgress !== options.maxProgress) dirty = true;
            taskInfo.maxProgress = options.maxProgress;
        }
        taskInfo.lastUpdateTime = Date.now();
        if (dirty) taskComponent.component.updateDOM(taskInfo);
    }
    function markAsStarted() {
        taskInfo.state = { stage: "started" };
        taskList.push(taskInfo);
        activeTaskList.push(taskInfo);
        const allTasksDiv = select(document, "div", "#all-tasks");
        allTasksDiv.append(taskComponent);
        console.log(taskList);
        console.log(activeTaskList);
    }
    function markAsFinishedSuccess(result: any) {
        taskInfo.state = { stage: "finished_success", result };
        activeTaskList.splice(activeTaskList.findIndex(x => x === taskInfo));
        taskComponent.component.updateDOM(taskInfo);
        console.log(taskList);
        console.log(activeTaskList);
    }
    function markAsFinishedError(error: any) {
        taskInfo.state = { stage: "finished_error", error };
        activeTaskList.splice(activeTaskList.findIndex(x => x === taskInfo));
        taskComponent.component.updateDOM(taskInfo);
        console.log(taskList);
        console.log(activeTaskList);
    }

    markAsStarted();
    try {
        const result = await taskFunc(updateStatus);
        markAsFinishedSuccess(result);
        return result;
    } catch (error) {
        markAsFinishedError(error);
        throw error;
    }
}