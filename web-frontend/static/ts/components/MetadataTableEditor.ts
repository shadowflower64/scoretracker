import { ComponentTemplate, place, select, selectAll } from "../Component.js";
import type { GenericMetadata, MetadataValue } from "../scoretracker/DataStructures.js";

export function getMetadataValueType(value: MetadataValue) {
    if (typeof value === "boolean") {
        return "boolean";
    } else if (typeof value === "number") {
        return "number";
    } else if (typeof value === "string") {
        return "string";
    } else {
        throw new Error(`invalid metadata value type: ${value} (type: ${typeof value})`);
    }
}

export const MetadataTableEditorRow = ComponentTemplate.named("metadata-table-editor-row", (f, params: { key: string, value: MetadataValue; }) => {
    const tr = select(f, "tr", "tr");
    const typeInput = select(f, "select", "#type");
    const keyInput = select(f, "input", "#key");
    const valueInput = select(f, "input", "#value");
    const deleteBtn = select(f, "button", "#delete-btn");
    const undeleteBtn = select(f, "button", "#undelete-btn");

    keyInput.value = params.key;
    if (typeof params.value === "boolean") {
        typeInput.value = "boolean";
        valueInput.type = "checkbox";
        valueInput.checked = params.value;
    } else if (typeof params.value === "number") {
        typeInput.value = "number";
        valueInput.type = "number";
        valueInput.value = params.value.toString();
    } else if (typeof params.value === "string") {
        typeInput.value = "string";
        valueInput.type = "text";
        valueInput.value = params.value;
    }

    typeInput.addEventListener("change", () => {
        if (typeInput.value === "string") {
            valueInput.type = "text";
        } else if (typeInput.value === "number") {
            valueInput.type = "number";
        } else if (typeInput.value === "boolean") {
            valueInput.type = "checkbox";
        } else {
            throw new Error(`invalid value: ${typeInput.value}`);
        }
    });
    deleteBtn.addEventListener("click", () => {
        tr.classList.add("deleted");
        typeInput.disabled = true;
        keyInput.disabled = true;
        valueInput.disabled = true;
    });
    undeleteBtn.addEventListener("click", () => {
        tr.classList.remove("deleted");
        typeInput.disabled = false;
        keyInput.disabled = false;
        valueInput.disabled = false;
    });
});

export const MetadataTableEditor = ComponentTemplate.named("metadata-table-editor", (f, params: { metadata: GenericMetadata; }) => {
    const addRowBtn = select(f, "button", "#add-row-btn");
    const tbody = select(f, "tbody", "tbody");
    addRowBtn.addEventListener("click", () => {
        tbody.append(MetadataTableEditorRow.create({ key: "newKey", value: "newValue" }));
    });
    function getDataFromRow(tr: HTMLTableRowElement): [key: string, value: MetadataValue] {
        const typeInput = select(tr, "select", "#type");
        const keyInput = select(tr, "input", "#key");
        const valueInput = select(tr, "input", "#value");
        const key = keyInput.value;
        if (typeInput.value === "boolean") {
            return [key, valueInput.checked];
        } else if (typeInput.value === "number") {
            return [key, valueInput.valueAsNumber];
        } else if (typeInput.value === "string") {
            return [key, valueInput.value];
        } else {
            throw new TypeError(`typeInput has an unknown value: ${typeInput.value}`);
        }
    }

    place(f, "rows", [
        ...Object.entries(params.metadata).map(([key, value]) => MetadataTableEditorRow.create({ key, value }))
    ]);
    return {
        getData: () => {
            const dataRows = selectAll(tbody, "tr", ".metadata-table-editor-row");
            return Object.fromEntries(dataRows.filter(tr => !tr.classList.contains("deleted")).map(getDataFromRow));
        }
    };
});