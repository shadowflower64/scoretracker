import { ComponentTemplate, place, select, selectAll } from "../Component.js";
import { UUID, UUID7 } from "../lib/uuid.js";

export const PerformanceTableEditorRow = ComponentTemplate.named("performance-table-editor-row", (f, params: { uuid: UUID; }) => {
    place(f, "uuid", params.uuid.toString());
});

export const PerformanceTableEditor = ComponentTemplate.named("performance-table-editor", (f, params: { performanceIds: UUID[]; }) => {
    const addRowBtn = select(f, "button", "#add-row-btn");
    const tbody = select(f, "tbody", "tbody");
    addRowBtn.addEventListener("click", () => {
        tbody.append(PerformanceTableEditorRow.create({ uuid: UUID7.generate() }));
    });

    place(f, "rows", [
        ...params.performanceIds.map((uuid) => PerformanceTableEditorRow.create({ uuid }))
    ]);

    function getDataFromRow(tr: HTMLTableRowElement): string {
        const uuidEl = select(tr, "code", "#uuid");
        return uuidEl.textContent || "";
    }

    return {
        getData: () => {
            const dataRows = selectAll(tbody, "tr", ".performance-table-editor-row");
            return [...dataRows.filter(tr => !tr.classList.contains("deleted")).map(getDataFromRow)];
        }
    };
});