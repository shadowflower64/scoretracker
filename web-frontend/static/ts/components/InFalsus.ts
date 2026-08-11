import type * as InFalsus from "../gen/types/in_falsus.schema.js";
import { ComponentTemplate, place, select } from "../Component.js";
import { Nanoseconds } from "../scoretracker/DataStructures.js";
import { sendRequestAsJSON } from "../Util.js";
import { UUID7 } from "../lib/uuid.js";
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
        await sendRequestAsJSON(`/api/match/${all.uuid}`, "PUT", all);
        const dialog = select(document, "dialog", "#edit-match-dialog-infalsus");
        dialog.close();
    });
});

export const EditMatchBtnInFalsus = ComponentTemplate.named("edit-match-btn-infalsus", (f, params: { match: InFalsus.Match; }) => {
    const btn = select(f, "button", "#btn");
    btn.addEventListener("click", () => showEditMatchDialogInFalsus(params.match));
});