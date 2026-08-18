type DirectReplacement = string | Node | DocumentFragment[];
type Replacement = DirectReplacement | ((i: number) => DirectReplacement);
type ReplacementMap = { [key: string]: Replacement; };
type CreateFn<P, I> = (fragment: DocumentFragment, params: P) => I;

export function select<K extends keyof HTMLElementTagNameMap>(parent: ParentNode, tag: K, selector: string): HTMLElementTagNameMap[K] {
    const result = parent.querySelector(selector);
    if (result === null) {
        throw new TypeError(`element '${selector}' not found`);
    }

    const tagName = tag.toLowerCase();
    const resultTagName = result.tagName.toLowerCase();
    if (resultTagName !== tagName) {
        throw new TypeError(`element '${selector}' is not a <${tagName}> (it's a <${resultTagName}>)`);
    }
    return result as HTMLElementTagNameMap[K];
}

export function selectAll<K extends keyof HTMLElementTagNameMap>(parent: ParentNode, tag: K, selector: string): HTMLElementTagNameMap[K][] {
    const result = parent.querySelectorAll(selector);
    const tagName = tag.toLowerCase();
    const values = result.values().filter((result) => {
        const resultTagName = result.tagName.toLowerCase();
        if (resultTagName !== tagName) {
            console.warn(`element '${selector}' is not a <${tagName}> (it's a <${resultTagName}>)`);
        }

        return resultTagName === tagName;
    });

    return [...values] as HTMLElementTagNameMap[K][];
}

export function selectAny(parent: ParentNode, selector: string): HTMLElement {
    const result = parent.querySelector(selector);
    if (result === null) {
        throw new TypeError(`element '${selector}' not found`);
    }
    return result as HTMLElement;
}

export function place<T extends Replacement>(fragment: DocumentFragment, key: string, replacement: T) {
    function replacePlaceholder(placeholder: Element, index: number) {
        let directReplacement;
        if (typeof replacement === "function") {
            directReplacement = replacement(index);
        } else {
            directReplacement = replacement;
        }

        if (typeof directReplacement === "string") {
            placeholder.replaceWith(document.createTextNode(directReplacement));
        } else if (Array.isArray(directReplacement)) {
            placeholder.replaceWith(...directReplacement);
        } else {
            placeholder.replaceWith(directReplacement);
        }
    }

    const placeholders = [
        ...fragment.querySelectorAll(`placeholder[name="${key}"]`),
        ...fragment.querySelectorAll(`.placeholder-${key}`)
    ];

    if (placeholders.length === 0) {
        throw new Error(`Placeholder with key '${key}' not found`);
    }
    placeholders.forEach(replacePlaceholder);
    return replacement;
}

export function placeAll(fragment: DocumentFragment, replacements: ReplacementMap) {
    Object.entries(replacements).forEach(([key, replacementCallbackOrElement]) => {
        place(fragment, key, replacementCallbackOrElement);
    });
}

export function onRemove(watchedNode: Node, destructor: () => void, fromParent: ParentNode = document.body) {
    let inDOM = fromParent.contains(watchedNode);
    const observer = new MutationObserver((mutations) => {
        if (fromParent.contains(watchedNode)) {
            if (!inDOM) {
                console.log("element inserted");
            }
            inDOM = true;
        } else {
            if (inDOM) {
                console.log("element removed");
                destructor();
            }
            inDOM = false;
        }

    });
    observer.observe(fromParent, { childList: true, subtree: true });
}

export type Component<I> = DocumentFragment & { component: I; };
export class ComponentTemplate<P = {}, I = undefined> {
    private constructor(readonly templateElement: HTMLTemplateElement, private readonly createFn: CreateFn<P, I>) { }

    static named<P = {}, I = undefined>(componentName: string, createFn: CreateFn<P, I>) {
        if (!("content" in document.createElement("template"))) {
            throw new Error(`browser does not support html <template> elements`);
        }

        const element = document.querySelector(`body > template[name="${componentName}"]`);
        if (element === null) {
            throw new Error(`template not found: ${componentName}`);
        }
        if (!("content" in element)) {
            throw new Error(`component not a <template>: ${componentName}`);
        }
        return new ComponentTemplate(element as HTMLTemplateElement, createFn);
    }

    create(params: P): Component<I> {
        const name = this.templateElement.getAttribute("name");
        console.log(`Creating component: '${name}'`, this.templateElement);

        const fragment = document.importNode(this.templateElement.content, true);
        const insides = this.createFn(fragment, params);

        // check for unused placeholders
        const unusedPlaceholders = fragment.querySelectorAll("placeholder, *[class^='placeholder-'], *[class*=' placeholder-']");
        if (unusedPlaceholders.length !== 0) {
            console.error("Found unused placeholders after creating component:", unusedPlaceholders);
        }

        const component = fragment as Component<I>;
        component.component = insides;
        return component;
    }
}
export type InsidesOfTemplate<TemplateType> = TemplateType extends ComponentTemplate<infer _, infer I> ? I : never;
export type ParamsOfTemplate<TemplateType> = TemplateType extends ComponentTemplate<infer P, infer _> ? P : never;
export type ComponentMadeFrom<TemplateType> = Component<InsidesOfTemplate<TemplateType>>;