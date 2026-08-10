// Source - https://stackoverflow.com/a/79789354
// Posted by MT0, modified by community. See post 'Timeline' for change history
// Retrieved 2026-02-27, License - CC BY-SA 4.0


/**
 * A Univerally Unique Identifier (UUID).
 */
export class UUID {
    static get VARIANT_APOLLO_NCS() { return 0b0; }
    static get VARIANT_RFC4122() { return 0b10; }
    static get VARIANT_COM() { return 0b110; }
    value;

    /**
     * Construct a UUID from the bytes array.
     *
     * @param {Uint8Array} bytes The 16-bytes of the UUID.
     */
    constructor(bytes: Uint8Array) {
        if (bytes.length !== 16) {
            throw new TypeError("'bytes' array has to be 16 bytes long");
        }
        this.value = bytes;
    }

    /**
     * Convert the
     * @param {boolean} [use_separator=true]
     * @returns String
     */
    toString(use_separator: boolean = true) {
        // @ts-ignore
        const str = this.value.toHex();
        if (use_separator) {
            return `${str.slice(0, 8)}-${str.slice(8, 12)}-${str.slice(12, 16)}-${str.slice(16, 20)}-${str.slice(20, 32)}`;
        }
        else {
            return str;
        }
    }

    /**
     * The four-bit UUID version number.
     *
     * This is bits 48-51 (zero-indexed) of the UUID.
     *
     * @readonly
     * @memberof UUID
     * @returns Number
     */
    get version() {
        // @ts-ignore
        return this.value[6] >> 4;
    }

    /**
     * The UUID variant bits.
     *
     * Note:
     *     (Legacy) Apollo NCS UUIDs have a single `0` variant bit (bit 64).
     *     RFC 4122/DCE 1.1 UUIDs have two `10` variant bits (bits 64-65).
     *     Microsoft COM/DCOM UUIDs have 3 `110` variant bits (bits 64-66).
     *
     * @readonly
     * @memberof UUID
     * @returns Number
     */
    get variant() {
        // @ts-ignore
        const bits = this.value[8] >> 5;
        if ((bits & 0b100) == 0) return UUID.VARIANT_APOLLO_NCS;
        if ((bits & 0b010) == 0) return UUID.VARIANT_RFC4122;
        return UUID.VARIANT_COM;
    }
}

/**
 * A version 4 UUID.
 */
export class UUID4 extends UUID {
    /**
     * Generate a version 4 UUID.
     *
     * @param {Number} [variant=1] The UUID version 4 variant which determines the values of
     *     bits 64-66. The default is variant `1`.
     *     * Variant `0` is a legacy Apollo NCS UUID (bit 64 is `0` and bits 65-66 are
     *       random).
     *     * Variant `1` is a RFC 4122/DCE 1.1 UUID (bits 64-65 are `10` and bit 66 is
     *       random).
     *     * Variant `2` is a Microsoft COM/DCOM (where bits 64-66 are `110`).
     * @returns UUID4
     */
    static generate(variant: number = 1) {
        const rand = crypto.getRandomValues(new Uint8Array(16));
        // Set version
        // @ts-ignore
        rand[6] = 0b01000000 | (rand[6] & 0b00001111);
        // Set variant
        switch (variant) {
            case 2:
                // @ts-ignore
                rand[8] = (UUID.VARIANT_COM << 5) | (rand[8] & 0b00011111);
                break;
            case 0:
                // @ts-ignore
                rand[8] = (UUID.VARIANT_APOLLO_NCS << 7) | (rand[8] & 0b01111111);
                break;
            default:
                // @ts-ignore
                rand[8] = (UUID.VARIANT_RFC4122 << 6) | (rand[8] & 0b00111111);
        }

        return new UUID4(rand);
    }
}

/**
 * A version 7 UUID.
 */
export class UUID7 extends UUID {
    /**
     * Generate a version 7 UUID.
     *
     * @returns UUID7
     */
    static generate() {
        const time = Date.now();
        // It appears to be faster to generate 16 bytes and overwrite with the time
        // than to generate 10 bytes and copy to another buffer containing the time.
        const rand = crypto.getRandomValues(new Uint8Array(16));
        // Set time
        rand[0] = Math.floor(time / 0x010000000000) % 256;
        rand[1] = Math.floor(time / 0x000100000000) % 256;
        rand[2] = Math.floor(time / 0x000001000000) % 256;
        rand[3] = Math.floor(time / 0x000000010000) % 256;
        rand[4] = Math.floor(time / 0x000000000100) % 256;
        rand[5] = time % 256;
        // Set version
        // @ts-ignore
        rand[6] = 0b01110000 | (rand[6] & 0b00001111);
        // Set variant
        // @ts-ignore
        rand[8] = (UUID.VARIANT_RFC4122 << 6) | (rand[8] & 0b00111111);

        return new UUID7(rand);
    }

    /**
     * The time component of the UUID.
     *
     * Note: This is stored in bits 0-47 (zero-indexed) of the UUID.
     *
     * @readonly
     * @memberof UUID7
     * @returns Date
     */
    get time() {
        return new Date(
            // @ts-ignore
            this.value[0] * 0x010000000000
            // @ts-ignore
            + this.value[1] * 0x000100000000
            // @ts-ignore
            + this.value[2] * 0x000001000000
            // @ts-ignore
            + this.value[3] * 0x000000010000
            // @ts-ignore
            + this.value[4] * 0x000000000100
            // @ts-ignore
            + this.value[5],
        );
    }
}
