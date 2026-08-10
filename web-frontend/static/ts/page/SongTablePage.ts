import { screenDiv } from "../app.js";
import { EditMatchBtnInFalsus } from "../components/InFalsus.js";
import { SongTable, SongTableRow } from "../components/SongTable.js";
import type { ALL_GAMES } from "../scoretracker/Games.js";
import { AbstractPage } from "./AbstractPage.js";


function makeSongTableRow(title: string, artist: string) {
    return SongTableRow.create({ title, artist, rowId: `${artist}-${title}` });
}

export class SongTablePage extends AbstractPage {
    open(): void {
        const songTable = SongTable.create({ gameId: this.gameId, rows: [makeSongTableRow("Slow Ride", "Foghat"), makeSongTableRow("Through the Fire and Flames", "Dragonforce")] });
        screenDiv.append(songTable);
        screenDiv.append(document.createElement("hr"));

        const editMatchBtn = EditMatchBtnInFalsus.create({});
        screenDiv.append(editMatchBtn);
    }
    constructor(protected gameId: keyof typeof ALL_GAMES) {
        super();
    }
}