/*
 * Your qualification function
 * The signature should never be changed
 *
 * The input type is `Component`
 * The return type is `Qualification`
 *
 * // The properties are derived from the fields in the Attributes panel
 * interface Properties {
 *   si: unknown;
 *   domain: unknown
 * }
 *
 * enum Kind {
 *   Standard,
 *   Credential
 * }
 *
 * interface Data {
 *   kind: Kind;
 *   properties: Properties;
 * }
 *
 * interface Code {
 *   format: string;
 *   code: string | null;
 * }
 *
 * interface Component {
 *   data: Data;
 *   parents: Component[]; // The parent's parents won't be available
 * }
 *
 * interface Qualification {
 *   result: 'success' | 'warning' | 'failure';
 *   message: string;
 * }
 */
async function qualification(component) {
  return {
    result: 'success',
    message: 'Component qualified'
 };
}
