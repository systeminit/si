import * as Y from 'yjs'

/**
 * Defines a range on text using relative positions that can be transformed back to
 * absolute positions. (https://docs.yjs.dev/api/relative-positions)
 */
export class YRange {
  /**
   * @param {Y.RelativePosition} yanchor
   * @param {Y.RelativePosition} yhead
   */
  constructor (yanchor, yhead) {
    this.yanchor = yanchor
    this.yhead = yhead
  }

  /**
   * @returns {any}
   */
  toJSON () {
    return {
      yanchor: Y.relativePositionToJSON(this.yanchor),
      yhead: Y.relativePositionToJSON(this.yhead)
    }
  }

  /**
   * @param {any} json
   * @return {YRange}
   */
  static fromJSON (json) {
    return new YRange(Y.createRelativePositionFromJSON(json.yanchor), Y.createRelativePositionFromJSON(json.yhead))
  }
}
