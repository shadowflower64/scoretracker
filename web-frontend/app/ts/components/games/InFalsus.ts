import { ComponentTemplate, place, select } from "../../Component.js";
import type { InFalsusMatchDetails } from "../../gen/types/in_falsus.schema.js";
import type { Match } from "../../scoretracker/DataStructures.js";
import { sendRequestAsJSON } from "../../Util.js";
import { commonMatchInfoFromParts, EditMatchDialogGenericPartBottom, EditMatchDialogGenericPartTop } from "../EditMatchForm.js";

export function showEditMatchDialogInFalsus(match: Match<InFalsusMatchDetails>) {
    const dialog = select(document, "dialog", "#edit-match-dialog-infalsus");
    dialog.innerHTML = "";
    dialog.append(EditMatchDialogInFalsus.create({ match }));
    dialog.showModal();
}

export const EditMatchDialogInFalsus = ComponentTemplate.named("edit-match-dialog-infalsus", (f, params: { match: Match<InFalsusMatchDetails>; }) => {
    const genericPartTop = EditMatchDialogGenericPartTop.create({ ...params.match });
    const genericPartBottom = EditMatchDialogGenericPartBottom.create({ ...params.match });

    place(f, "generic-part-top", genericPartTop);
    place(f, "generic-part-bottom", genericPartBottom);

    const form = select(f, "form", "form");
    form.addEventListener("submit", async () => {
        const common = commonMatchInfoFromParts(genericPartTop, genericPartBottom);
        const all: Match<InFalsusMatchDetails> = {
            ...common,
            details: {},
        };
        await sendRequestAsJSON(`/api/match/${all.match_uuid}`, "PUT", all);
        const dialog = select(document, "dialog", "#edit-match-dialog-infalsus");
        dialog.close();
    });
});

export const EditMatchBtnInFalsus = ComponentTemplate.named("edit-match-btn-infalsus", (f, params: { match: Match<InFalsusMatchDetails>; }) => {
    const btn = select(f, "button", "#btn");
    btn.addEventListener("click", () => showEditMatchDialogInFalsus(params.match));
});