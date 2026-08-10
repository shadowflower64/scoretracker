import { ComponentTemplate, place, select, selectAll } from "../Component.js";
import { UUID, UUID7 } from "../uuid.js";

export const ProofTableEditorRow = ComponentTemplate.named("proof-table-editor-row", (f, params: { uuid: UUID; }) => {
    place(f, "uuid", params.uuid.toString());
});

export const ProofTableEditor = ComponentTemplate.named("proof-table-editor", (f, params: { performanceIds: UUID[]; }) => {
    const addRowBtn = select(f, "button", "#add-row-btn");
    const tbody = select(f, "tbody", "tbody");
    addRowBtn.addEventListener("click", () => {
        tbody.append(ProofTableEditorRow.create({ uuid: UUID7.generate() }));
    });

    place(f, "rows", [
        ...params.performanceIds.map((uuid) => ProofTableEditorRow.create({ uuid }))
    ]);

    function getDataFromRow(tr: HTMLTableRowElement): string {
        const uuidEl = select(tr, "code", "#uuid");
        return uuidEl.textContent || "";
    }

    return {
        getData: () => {
            const dataRows = selectAll(tbody, "tr", ".proof-table-editor-row");
            return [...dataRows.filter(tr => !tr.classList.contains("deleted")).map(getDataFromRow)];
        }
    };
});