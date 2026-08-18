import { ComponentTemplate, place, select, type ComponentMadeFrom } from "../Component.js";
import { Nanoseconds, type CommonMatchInfo, type MatchMetadata, type NsTimestamp } from "../scoretracker/DataStructures.js";
import { MetadataTableEditor } from "./MetadataTableEditor.js";
import { PerformanceTableEditor } from "./PerformanceTableEditor.js";
import { ProofTableEditor } from "./ProofTableEditor.js";


export function commonMatchInfoFromParts(genericPartTop: ComponentMadeFrom<typeof EditMatchDialogGenericPartTop>, genericPartBottom: ComponentMadeFrom<typeof EditMatchDialogGenericPartBottom>): CommonMatchInfo {
    return {
        ...genericPartTop.component.getFormValues(),
        ...genericPartBottom.component.getFormValues()
    };
}


export const EditMatchDialogGenericPartTop = ComponentTemplate.named("edit-match-dialog-generic-part-top", (f, params: { uuid: string, timestamp: NsTimestamp, song_id: string; }) => {
    const uuid = select(f, "input", "#uuid");
    const timestampDate = select(f, "input", "#timestamp-date");
    const timestampTime = select(f, "input", "#timestamp-time");
    const timestampNanos = select(f, "input", "#timestamp-nanos");
    const songId = select(f, "input", "#song-id");

    uuid.value = params.uuid;
    const [date, nanosFrac] = Nanoseconds.dateParts(params.timestamp);
    timestampDate.valueAsDate = date;
    timestampTime.valueAsDate = date;
    timestampNanos.valueAsNumber = nanosFrac;
    songId.value = params.song_id;

    return {
        getFormValues() {
            const timestamp = Nanoseconds.fromMillisParts(Date.parse(`${timestampDate.value}T${timestampTime.value}`), timestampNanos.valueAsNumber);
            console.log(timestamp);
            return {
                uuid: uuid.value,
                timestamp,
                song_id: songId.value
            };
        }
    };
});
export const EditMatchDialogGenericPartBottom = ComponentTemplate.named("edit-match-dialog-generic-part-bottom", (f, params: { proof: string[], comment?: string | null, metadata: MatchMetadata; }) => {
    const comment = select(f, "textarea", "#comment");
    // const performanceTableEditor = place(f, "performance-table-editor", PerformanceTableEditor.create({ performanceIds: [] })); // TODO
    const proofTableEditor = place(f, "proof-table-editor", ProofTableEditor.create({ performanceIds: [] }));
    const metadataTableEditor = place(f, "metadata-table-editor", MetadataTableEditor.create({ metadata: params.metadata }));
    return {
        metadataTableEditor: metadataTableEditor.component,
        getFormValues() {
            return {
                proof: proofTableEditor.component.getData(),
                comment: comment.value,
                metadata: metadataTableEditor.component.getData()
            };
        }
    };
});