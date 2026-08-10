import type { BigintNsTimestamp, NsTimestamp } from "./scoretracker/DataStructures.js";

export function unwrap<T>(foo: T | null | undefined): T {
    if (foo === null) throw new TypeError("element is null");
    if (foo === undefined) throw new TypeError("element is undefined");
    return foo;
}

export function sleep(ms: number): Promise<void> {
    return new Promise((resolve, _reject) => {
        setTimeout(() => {
            resolve();
        }, ms);
    });
}

export function betterJSONStringify(body: any): string {
    /// TODO: this is just really bad.
    /// the main problem is that when deserializing, there is no way to distinguish
    /// any normal number, which should be deserialized as a number,
    /// from a timestamp, which should be deserialized as a bigint,
    /// without guessing or hardcoding the types.
    /// there is also no way to distinguish normal strings from bigint strings
    /// in the same way.
    ///
    /// changing the API to count nanoseconds and seconds separately,
    /// would give enough precision to properly use the numbers
    /// in javascript.
    /// ...which is exactly what Rust does by default already.
    ///
    /// alternatively, it is possible to parse all numbers in json as bigints,
    /// and then preserve them as bigints if they are larger than `Number.MAX_SAFE_INTEGER`.
    /// that is still a guess though... and now every `number` type can just be a `bigint` instead,
    /// and every `bigint` can be a `number`.
    ///
    /// also serde_json doesn't want to work with i128 numbers well at all
    ///
    /// https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/BigInt#use_within_json
    /// https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/JSON#using_json_numbers
    return JSON.stringify(body, (_key, value) => {
        if (typeof value === "bigint") {
            console.log("serializing bigint:", value);
            // @ts-ignore
            return JSON.rawJSON(value.toString());
        }
        return value;
    });
}

export function betterJSONParse(text: string): any {
    // @ts-ignore
    return JSON.parse(text, (_key, value, context) => {
        if (typeof value === "number" && (value > Number.MAX_SAFE_INTEGER || value < Number.MIN_SAFE_INTEGER)) {
            try {
                const trueValue = BigInt(context.source);
                console.log("deserializing as bigint (number value is outside of integer range):", value, "->", trueValue);
                return trueValue;
            } catch {
                return value;
            }
        }
        return value;
    });
}

// testing 
console.log(betterJSONStringify({ abc: 123456789012345678901, def: 123456789012345678901n }));
console.log(betterJSONParse(`{ "abc": 123456789, "def": 123456789012345678901 }`));

export async function fetchText(url: string): Promise<string> {
    const res = await fetch(url);
    return await res.text();
}

export async function fetchJSON(url: string): Promise<any> {
    const res = await fetch(url);
    return await res.json();
}

export function sendRequest(url: string, method: "GET" | "POST" | "PUT" = "POST", body: string | null = null): Promise<string> {
    return new Promise((resolve, reject) => {
        const xhr = new XMLHttpRequest();
        xhr.open(method, url, true);
        xhr.setRequestHeader("Content-Type", "application/json");
        xhr.onload = () => resolve(xhr.responseText);
        xhr.onerror = reject;
        xhr.send(body);
    });
}

export function sendRequestAsJSON(url: string, method: "GET" | "POST" | "PUT" = "POST", body: any): Promise<string> {
    return sendRequest(url, method, betterJSONStringify(body));
}
