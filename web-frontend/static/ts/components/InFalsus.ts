import { ComponentTemplate, place, select } from "../Component.js";
import type * as InFalsus from "../gen/types/in_falsus.schema.js";
import { Nanoseconds } from "../scoretracker/DataStructures.js";
import { sendRequestAsJSON } from "../Util.js";
import { UUID7 } from "../uuid.js";
import { commonMatchInfoFromParts, EditMatchDialogGenericPartBottom, EditMatchDialogGenericPartTop } from "./EditMatchForm.js";

export function showEditMatchDialogInFalsus(match: InFalsus.Match) {
    const dialog = select(document, "dialog", "#edit-match-dialog-infalsus");
    dialog.innerHTML = "";
    dialog.append(EditMatchDialogInFalsus.create({ match }));
    dialog.showModal();
}

export const EditMatchDialogInFalsus = ComponentTemplate.named("edit-match-dialog-infalsus", (f, params: { match: InFalsus.Match; }) => {
    const genericPartTop = EditMatchDialogGenericPartTop.create({ ...params.match });
    const genericPartBottom = EditMatchDialogGenericPartBottom.create({ ...params.match });

    place(f, "generic-part-top", genericPartTop);
    place(f, "generic-part-bottom", genericPartBottom);

    const form = select(f, "form", "form");
    form.addEventListener("submit", async () => {
        const common = commonMatchInfoFromParts(genericPartTop, genericPartBottom);
        const all: InFalsus.Match = common;
        console.log("all", all);

        await sendRequestAsJSON(`/api/match/${all.uuid}`, "PUT", all);

        const dialog = select(document, "dialog", "#edit-match-dialog-infalsus");
        dialog.close();
    });
});
export const EditMatchBtnInFalsus = ComponentTemplate.named("edit-match-btn-infalsus", (f, params) => {
    const btn = select(f, "button", "#btn");
    btn.addEventListener("click", () => showEditMatchDialogInFalsus({
        uuid: UUID7.generate().toString(),
        timestamp: Nanoseconds.fromMillisParts(Date.now(), 123456789),
        song_id: "xi-freedom_dive",
        performance_ids: [],
        proof: [],
        comment: "Example user comment",
        metadata: { abc: "def", ghi: 123, jkl: true }
    }));
});