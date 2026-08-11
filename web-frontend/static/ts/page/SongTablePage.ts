import { screenDiv } from "../main.js";
import { EditMatchBtnInFalsus } from "../components/games/InFalsus.js";
import { SongTable, SongTableRow } from "../components/SongTable.js";
import { UUID7 } from "../lib/uuid.js";
import { Nanoseconds } from "../scoretracker/DataStructures.js";
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

        const editMatchBtn = EditMatchBtnInFalsus.create({
            // TODO: this is just an example button
            match: {
                uuid: UUID7.generate().toString(),
                timestamp: Nanoseconds.fromMillisParts(Date.now(), 123456789),
                song_id: "xi-freedom_dive",
                performance_ids: [],
                proof: [],
                comment: "Example user comment",
                metadata: { abc: "def", ghi: 123, jkl: true }
            }
        });
        screenDiv.append(editMatchBtn);
    }
    constructor(protected gameId: keyof typeof ALL_GAMES) {
        super();
    }
}