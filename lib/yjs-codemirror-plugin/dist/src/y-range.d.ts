/**
 * Defines a range on text using relative positions that can be transformed back to
 * absolute positions. (https://docs.yjs.dev/api/relative-positions)
 */
export class YRange {
    /**
     * @param {any} json
     * @return {YRange}
     */
    static fromJSON(json: any): YRange;
    /**
     * @param {Y.RelativePosition} yanchor
     * @param {Y.RelativePosition} yhead
     */
    constructor(yanchor: Y.RelativePosition, yhead: Y.RelativePosition);
    yanchor: Y.RelativePosition;
    yhead: Y.RelativePosition;
    /**
     * @returns {any}
     */
    toJSON(): any;
}
import * as Y from "yjs";
