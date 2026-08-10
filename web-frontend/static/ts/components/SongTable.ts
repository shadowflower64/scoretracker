import { ComponentTemplate, place, select, selectAny } from "../Component.js";

export const SongTable = ComponentTemplate.named("song-table", (f, params: { gameId: string, rows: DocumentFragment[] }) => {
    const songTableBody = select(f, "tbody", ".song-table-body");
    const addRowButton = select(f, "button", ".add-row-button");
    const addRowCount = select(f, "input", ".add-row-count");
    const removeEmptyRowsButton = select(f, "button", ".remove-empty-rows-button");

    addRowButton.addEventListener("click", () => {
        const count = addRowCount.valueAsNumber;
        if (Number.isNaN(count)) {
            throw new Error("inputted value is not a number");
        }
        for (let i = 0; i < count; i++) {
            const rowId = `newRow${i}`;
            console.log("adding: ", rowId);
            songTableBody.appendChild(SongTableRow.create({ rowId: rowId, artist: "", title: "" }));
            console.log("added: ", rowId);
        }
    });
    removeEmptyRowsButton.addEventListener("click", () => {
        songTableBody.querySelectorAll("tr").forEach(tr => {
            if (tr.querySelector<HTMLInputElement>(".artist-input")?.value === "" && tr.querySelector<HTMLInputElement>(".title-input")?.value === "") {
                tr.remove();
            }
        });
    });

    place(f, "game-id", params.gameId);
    place(f, "rows", params.rows);
});

export const SongTableRow = ComponentTemplate.named("song-table-row", (f, params: { rowId: string, artist: string, title: string }) => {
    const self = selectAny(f, "tr");
    const artistInput = select(f, "input", ".artist-input");
    const titleInput = select(f, "input", ".title-input");
    const removeRowButton = select(f, "button", ".remove-row-button");

    artistInput.placeholder = params.artist;
    artistInput.value = params.artist;
    titleInput.placeholder = params.title;
    titleInput.value = params.title;
    removeRowButton.addEventListener("click", () => {
        console.log("removing: ", params.rowId);
        self.remove();
        console.log("removed: ", params.rowId);
    });
});