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
    return sendRequest(url, method, JSON.stringify(body));
}
