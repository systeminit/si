import { parseConnectionAnnotation } from "@si/ts-lib";
import Joi from "joi";
import { Debug } from "./debug";

const debug = Debug("langJs:asset_builder");

export type ValueFromKind = "inputSocket" | "outputSocket" | "prop";

export interface ValueFrom {
  kind: ValueFromKind;
  socket_name?: string;
  prop_path?: string[];
}

export interface IValueFromBuilder {
  setKind(kind: ValueFromKind): this;

  setSocketName(name: string): this;

  setPropPath(path: string[]): this;

  build(): ValueFrom;
}

/**
 * Gets a value from a socket or prop
 *
 * @example
 * const value = new ValueFromBuilder()
 *  .setKind("prop")
 *  .setPropPath(["root", "si", "name"])
 *  .build()
 */
export class ValueFromBuilder implements IValueFromBuilder {
  valueFrom = <ValueFrom>{};

  constructor() {
    this.valueFrom = <ValueFrom>{};
  }

  /**
   * The type of the builder
   *
   * @param kind {string} [inputSocket | outputSocket | prop]
   *
   * @returns this
   *
   * @example
   * .setKind("prop")
   */
  setKind(kind: ValueFromKind): this {
    this.valueFrom.kind = kind;
    return this;
  }

  /**
   * Specify the socket name if using an inputSocket or outputSocket
   *
   * @param {string} name
   *
   * @returns this
   *
   * @example
   * .setSocketName("Region")
   */
  setSocketName(name: string): this {
    if (
      this.valueFrom.kind !== "inputSocket"
      && this.valueFrom.kind !== "outputSocket"
    ) {
      return this;
    }

    this.valueFrom.socket_name = name;
    return this;
  }

  /**
   * Specify the prop path if using a prop
   *
   * @param {string[]} path - a list of strings that represent the path to the prop
   *
   * @returns this
   *
   * @example
   *  .setPropPath(["root", "si", "name"])
   */
  setPropPath(path: string[]): this {
    if (this.valueFrom.kind !== "prop") {
      return this;
    }

    this.valueFrom.prop_path = path;
    return this;
  }

  /**
   * Build the object
   *
   * @example
   *  .build()
   */
  build(): ValueFrom {
    return this.valueFrom;
  }
}

export type SocketDefinitionArityType = "many" | "one";

export interface SocketDefinition {
  name: string;
  arity: SocketDefinitionArityType;
  connectionAnnotations: string;
  uiHidden?: boolean;
  valueFrom?: ValueFrom;
}

export interface ISocketDefinitionBuilder {
  setName(name: string): this;

  setArity(arity: SocketDefinitionArityType): this;

  setConnectionAnnotation(annotation: string): this;

  setUiHidden(hidden: boolean): this;

  setValueFrom(valueFrom: ValueFrom): this;

  build(): SocketDefinition;
}

/**
 * Defines an input or output socket for passing values between components
 *
 * @example
 * const regionSocket = new SocketDefinitionBuilder()
 *  .setName("Region")
 *  .setArity("one")
 *  .build();
 */
export class SocketDefinitionBuilder implements ISocketDefinitionBuilder {
  socket = <SocketDefinition>{};
  connectionAnnotations: string[] = [];

  constructor() {
    this.socket = <SocketDefinition>{};
  }

  /**
   * Build the object
   *
   * @example
   *  .build()
   */
  build(): SocketDefinition {
    if (!this.socket.name) {
      throw new Error("Name is required for socket");
    }

    this.connectionAnnotations.push(this.socket.name.toLowerCase());

    this.socket.connectionAnnotations = JSON.stringify(
      this.connectionAnnotations.map((a) => a.toLowerCase().trim()),
    );

    return this.socket;
  }

  /**
   * Specify the number of connections the socket can support
   *
   * @param {string} arity - [one | many]
   *
   * @returns this
   *
   * @example
   *  .setArity("one")
   */
  setArity(arity: SocketDefinitionArityType): this {
    this.socket.arity = arity;
    return this;
  }

  /**
   * Add a field to the connection annotations array for the socket.
   * The input should be sequence of word chars (\w regex matcher), optionally
   * followed by any `<identifier>`, which makes it a supertype of `identifier`.
   * This can be repeated recursively as many times as necessary (see example).
   * At socket connecting time an *input* socket can receive a connection of any
   * *output* socket that has a compatible connection annotation.
   *
   * e.g. An input socket with the `Port<string>` connection
   * annotation can receive a
   * connection from an output socket with the `Docker<Port<string>>` annotation,
   * but not one with just `string`.
   *
   * The socket's name is always one of the connection annotations.
   *
   * @param {string} annotation
   *
   * @returns this
   *
   * @example
   *  .setConnectionAnnotation("EC2<IAM<string>>")
   */
  setConnectionAnnotation(annotation: string): this {
    // Throws if not able to match annotation
    parseConnectionAnnotation(annotation);

    this.connectionAnnotations.push(annotation);
    return this;
  }

  /**
   * The name of the socket. Note that this will be used to connect sockets
   * and to reference the socket within the asset.
   *
   * @param {string} name
   *
   * @returns this
   *
   * @example
   *  .setName("Subnet ID")
   */
  setName(name: string): this {
    this.socket.name = name;
    return this;
  }

  /**
   * Should this socket show in the UI. Note that the socket can still be connected when the component is placed in a frame.
   *
   * @param {boolean} hidden
   *
   * @returns this
   *
   * @example
   *  .setName("Subnet ID")
   */
  setUiHidden(hidden: boolean): this {
    this.socket.uiHidden = hidden;
    return this;
  }

  /**
   * Set the value of this socket using a ValueFromBuilder
   *
   * @param {ValueFrom} valueFrom
   *
   * @returns this
   *
   * @example
   *  .setValueFrom(new ValueFromBuilder()
   *    .setKind("inputSocket")
   *    .setSocketName("Region")
   *    .build())
   */
  setValueFrom(valueFrom: ValueFrom): this {
    this.socket.valueFrom = valueFrom;
    return this;
  }
}

export type ValidationKind =
  | "customValidation"
  | "integerIsBetweenTwoIntegers"
  | "integerIsNotEmpty"
  | "stringEquals"
  | "stringHasPrefix"
  | "stringInStringArray"
  | "stringIsHexColor"
  | "stringIsNotEmpty"
  | "stringIsValidIpAddr";

export interface Validation {
  kind: ValidationKind;
  funcUniqueId?: Record<string, unknown>;
  lowerBound?: number;
  upperBound?: number;
  expected?: string[];
  displayExpected?: boolean;
}

export interface IValidationBuilder {
  setKind(kind: ValidationKind): this;

  addFuncUniqueId(key: string, value: unknown): this;

  setLowerBound(value: number): this;

  setUpperBound(value: number): this;

  addExpected(expected: string): this;

  setDisplayExpected(display: boolean): this;

  build(): Validation;
}

/**
 * Validates a prop using a function or from a list of common validations
 *
 * @example
 * const validation = new ValidationBuilder()
 *  .setKind("stringIsNotEmpty")
 *  .build()
 */
export class ValidationBuilder implements IValidationBuilder {
  validation = <Validation>{};

  constructor() {
    this.validation = <Validation>{};
  }

  addFuncUniqueId(key: string, value: unknown): this {
    if (this.validation.kind !== "customValidation") {
      return this;
    }

    if (!this.validation.funcUniqueId) {
      this.validation.funcUniqueId = {};
    }

    this.validation.funcUniqueId[key] = value;
    return this;
  }

  /**
   * Build the object
   *
   * @example
   *  .build()
   */
  build(): Validation {
    return this.validation;
  }

  setDisplayExpected(display: boolean): this {
    if (this.validation.kind !== "stringInStringArray") {
      return this;
    }

    this.validation.displayExpected = display;
    return this;
  }

  addExpected(expected: string): this {
    if (
      this.validation.kind !== "stringEquals"
      && this.validation.kind !== "stringHasPrefix"
      && this.validation.kind !== "stringInStringArray"
    ) {
      return this;
    }

    if (!this.validation.expected) {
      this.validation.expected = [];
    }

    this.validation.expected.push(expected);
    return this;
  }

  setLowerBound(value: number): this {
    if (this.validation.kind !== "integerIsBetweenTwoIntegers") {
      return this;
    }
    this.validation.lowerBound = value;
    return this;
  }

  /**
   * The type of validation
   *
   * @param kind {string} [customValidation | integerIsBetweenTwoIntegers | integerIsNotEmpty  | stringEquals  | stringHasPrefix  | stringInStringArray  | stringIsHexColor  | stringIsNotEmpty  | stringIsValidIpAddr]
   *
   * @returns this
   *
   * @example
   * .setKind("integerIsNotEmpty")
   */
  setKind(kind: ValidationKind): this {
    this.validation.kind = kind;
    return this;
  }

  setUpperBound(value: number): this {
    if (this.validation.kind !== "integerIsBetweenTwoIntegers") {
      return this;
    }
    this.validation.upperBound = value;
    return this;
  }
}

export type PropWidgetDefinitionKind =
  | "array"
  | "checkbox"
  | "codeEditor"
  | "color"
  | "comboBox"
  | "header"
  | "map"
  | "password"
  | "secret"
  | "select"
  | "text"
  | "textArea";

export interface Option {
  label: string;
  value: string;
}

export interface PropWidgetDefinition {
  kind: PropWidgetDefinitionKind;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  options: Option[];
}

export interface IPropWidgetDefinitionBuilder {
  setKind(kind: string): this;

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  addOption(key: string, value: string): this;

  build(): PropWidgetDefinition;
}

/**
 * Create a widget for interacting with a prop that is displayed in the modelling view.
 *
 * @example
 * const validation = new PropWidgetDefinitionBuilder()
 *  .setKind("text")
 *  .build()
 */
export class PropWidgetDefinitionBuilder
implements IPropWidgetDefinitionBuilder {
  propWidget = <PropWidgetDefinition>{};

  constructor() {
    this.propWidget = <PropWidgetDefinition>{};
  }

  /**
   * The type of widget
   *
   * @param kind {PropWidgetDefinitionKind} [array | checkbox | color | comboBox | header | map | select | text | textArea | codeEditor | password]
   *
   * @returns this
   *
   * @example
   * .setKind("color")
   */
  setKind(kind: PropWidgetDefinitionKind): this {
    this.propWidget.kind = kind;
    return this;
  }

  /**
   * Add an option when using a comboBox
   *
   * @param {string} key - the value displayed in the comboBox
   * @param {string} value - the value the prop is set to
   *
   * @returns this
   *
   * @example
   * .setOption("us-east-2 - US East (Ohio)", "us-east-2")
   */
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  addOption(key: string, value: string): this {
    if (!this.propWidget.options) {
      this.propWidget.options = [];
    }

    this.propWidget.options.push(<Option>{
      label: key,
      value,
    });
    return this;
  }

  /**
   * Build the object
   *
   * @example
   *  .build()
   */
  build(): PropWidgetDefinition {
    return this.propWidget;
  }
}

export interface MapKeyFunc {
  key: string;
  valueFrom?: ValueFrom;
}

export interface IMapKeyFuncBuilder {
  setKey(key: string): this;

  setValueFrom(valueFrom: ValueFrom): this;

  build(): MapKeyFunc;
}

/**
 * Used to add a value to a map
 *
 * @example
 *  const mapButton = new MapKeyFuncBuilder()
 *    .setKey("Name")
 *    .build()
 */
export class MapKeyFuncBuilder implements IMapKeyFuncBuilder {
  mapKeyFunc = <MapKeyFunc>{};

  constructor() {
    this.mapKeyFunc = <MapKeyFunc>{};
  }

  /**
   * Build the object
   *
   * @example
   *  .build()
   */
  build(): MapKeyFunc {
    return this.mapKeyFunc;
  }

  /**
   * Set the value of the key for the map entry
   *
   * @param {string} key - the name of the key
   *
   * @returns this
   *
   * @example
   *  .setKey("Name")
   */
  setKey(key: string): this {
    this.mapKeyFunc.key = key;
    return this;
  }

  /**
   * Set the value of this key from a ValueFromBuilder
   *
   * @param {ValueFrom} valueFrom
   *
   * @returns this
   *
   * @example
   *  .setValueFrom(new ValueFromBuilder()
   *    .setKind("prop")
   *    .setPropPath(["root", "si", "name"])
   *    .build())
   */
  setValueFrom(valueFrom: ValueFrom): this {
    this.mapKeyFunc.valueFrom = valueFrom;
    return this;
  }
}

export type SiPropValueFromDefinitionKind =
  | "color"
  | "name"
  | "resourcePayload";

export interface SiPropValueFromDefinition {
  kind: SiPropValueFromDefinitionKind;
  valueFrom: ValueFrom;
}

export interface ISiPropValueFromDefinitionBuilder {
  setKind(kind: SiPropValueFromDefinitionKind): this;

  setValueFrom(valueFrom: ValueFrom): this;

  build(): SiPropValueFromDefinition;
}

export class SiPropValueFromDefinitionBuilder
implements ISiPropValueFromDefinitionBuilder {
  definition = <SiPropValueFromDefinition>{};

  constructor() {
    this.definition = <SiPropValueFromDefinition>{};
  }

  /**
   * Build the object
   *
   * @example
   *  .build()
   */
  build(): SiPropValueFromDefinition {
    return this.definition;
  }

  setKind(kind: SiPropValueFromDefinitionKind): this {
    this.definition.kind = kind;
    return this;
  }

  setValueFrom(valueFrom: ValueFrom): this {
    this.definition.valueFrom = valueFrom;
    return this;
  }
}

export type PropDefinitionKind =
  | "array"
  | "boolean"
  | "integer"
  | "map"
  | "object"
  | "string";

export interface PropDefinition {
  name: string;
  kind: PropDefinitionKind;
  docLinkRef?: string;
  docLink?: string;
  documentation?: string;
  children?: PropDefinition[];
  entry?: PropDefinition;
  widget?: PropWidgetDefinition;
  valueFrom?: ValueFrom;
  hidden?: boolean;
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  defaultValue?: any;
  validations?: Validation[];
  validationFormat: string; // A JSON.stringify()-ed Joi.Descriptor
  mapKeyFuncs?: MapKeyFunc[];
}

export interface IPropBuilder {
  setName(name: string): this;

  setKind(kind: PropDefinitionKind): this;

  setDocLinkRef(ref: string): this;

  setDocumentation(ref: string): this;

  setDocLink(link: string): this;

  addChild(child: PropDefinition): this;

  setEntry(entry: PropDefinition): this;

  setWidget(widget: PropWidgetDefinition): this;

  setValueFrom(valueFrom: ValueFrom): this;

  setHidden(hidden: boolean): this;

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  setDefaultValue(value: any): this;

  addValidation(validation: Validation): this;

  setValidationFormat(format: Joi.Schema): this;

  addMapKeyFunc(func: MapKeyFunc): this;

  build(): PropDefinition;
}

/**
 * Creates a prop to attach values to an asset
 *
 * @example
 *  const propName = new PropBuilder()
 *   .setName("name")
 *   .setKind("string")
 *   .setDocumentation("This is the documentation for the prop")
 *   .setWidget(new PropWidgetDefinitionBuilder().setKind("text").build())
 *  .build();
 */
export class PropBuilder implements IPropBuilder {
  prop = <PropDefinition>{};

  constructor() {
    this.prop = <PropDefinition>{
      validationFormat: JSON.stringify(Joi.any().describe()),
    };
  }

  /**
   * Adds a child to an object type prop
   *
   * @param {PropDefinition} child
   *
   * @returns this
   *
   * @example
   *   .addChild(new PropBuilder()
   *     .setKind("string")
   *     .setName("sweetChildProp")
   *     .setWidget(new PropWidgetDefinitionBuilder().setKind("text").build())
   *     .build())
   */
  addChild(child: PropDefinition): this {
    if (this.prop.kind !== "object") {
      throw new Error("addChild can only be called on props that are objects");
    }

    if (!this.prop.children) {
      this.prop.children = [];
    }

    this.prop.children.push(child);
    return this;
  }

  /**
   * Adds an entry to array or map type props
   *
   * @param {PropDefinition} entry
   *
   * @returns this
   *
   * @example
   *   .setEntry(new PropBuilder()
   *     .setKind("string")
   *     .setName("iamanentryprop")
   *     .setWidget(new PropWidgetDefinitionBuilder().setKind("text").build())
   *     .build())
   */
  setEntry(entry: PropDefinition): this {
    if (this.prop.kind !== "array" && this.prop.kind !== "map") {
      throw new Error(
        "setEntry can only be called on prop that are arrays or maps",
      );
    }

    this.prop.entry = entry;
    return this;
  }

  /**
   * Add a button for putting entries into maps
   *
   * @param {MapKeyFunc} func
   *
   * @returns this
   *
   * @example
   *  .addMapKeyFunc(new MapKeyFuncBuilder()
   *    .setKey("Name")
   *    .build()
   */
  addMapKeyFunc(func: MapKeyFunc): this {
    if (!this.prop.mapKeyFuncs) {
      this.prop.mapKeyFuncs = [];
    }
    this.prop.mapKeyFuncs.push(func);
    return this;
  }

  /**
   * Add functions to validate the value of the prop
   *
   * @param {Validation} validation
   *
   * @returns this
   *
   * @example
   * .addValidation(new ValidationBuilder()
   *  .setKind("stringIsNotEmpty")
   *  .build())
   */
  addValidation(validation: Validation): this {
    if (!this.prop.validations) {
      this.prop.validations = [];
    }
    this.prop.validations.push(validation);
    return this;
  }

  /**
   * Add joi validation schema to this prop
   *
   * @returns this
   *
   * @example
   * .setValidationFormat(Joi.string().required())
   * @param format {Joi.Schema} - A joi schema object
   */
  setValidationFormat(format: Joi.Schema): this {
    try {
      this.prop.validationFormat = JSON.stringify(format.describe());
    } catch (e) {
      const message = e instanceof Error ? e.message : "unknown";
      throw Error(`Error compiling validation format: ${message}`);
    }

    return this;
  }

  /**
   * Build the object
   *
   * @example
   *  .build()
   */
  build(): PropDefinition {
    return this.prop;
  }

  /**
   * Set a value to be automatically populated in the prop
   *
   * @param {any} value
   *
   * @returns this
   *
   * @example
   * .setDefaultValue("cats")
   */
  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  setDefaultValue(value: any): this {
    this.prop.defaultValue = value;
    return this;
  }

  /**
   * Set a link to external documentation that will appear beneath the prop
   *
   * @param {string} link
   *
   * @returns this
   *
   * @example
   *  .setDocLink("https://www.systeminit.com/")
   */
  setDocLink(link: string): this {
    this.prop.docLink = link;
    return this;
  }

  /**
   * Sets inline documentation for the prop
   *
   * @param {string} docs
   *
   * @returns this
   *
   * @example
   *  .setDocumentation("This is documentation for the prop")
   */
  setDocumentation(docs: string): this {
    this.prop.documentation = docs;
    return this;
  }

  setDocLinkRef(ref: string): this {
    this.prop.docLinkRef = ref;
    return this;
  }

  /**
   * Whether the prop should be displayed in th UI or not
   *
   * @param {boolean} hidden
   *
   * @returns this
   *
   * @example
   *  .setHidden(true)
   */
  setHidden(hidden: boolean): this {
    this.prop.hidden = hidden;
    return this;
  }

  /**
   * The type of the prop
   *
   * @param kind {PropDefinitionKind} [array | boolean | integer | map | object | string]
   *
   * @returns this
   *
   * @example
   * .setKind("text")
   */
  setKind(kind: PropDefinitionKind): this {
    this.prop.kind = kind;
    return this;
  }

  /**
   * The prop name. This will appear in the model UI
   *
   * @param {string} name - the name of the prop
   *
   * @returns this
   *
   * @example
   * .setName("Region")
   */
  setName(name: string): this {
    this.prop.name = name;
    return this;
  }

  /**
   * Set the value of this prop using a ValueFromBuilder
   *
   * @param {ValueFrom} valueFrom
   *
   * @returns this
   *
   * @example
   *  .setValueFrom(new ValueFromBuilder()
   *    .setKind("inputSocket")
   *    .setSocketName("Region")
   *    .build())
   */
  setValueFrom(valueFrom: ValueFrom): this {
    this.prop.valueFrom = valueFrom;
    return this;
  }

  /**
   * The type of widget for the prop, determing how it is displayed in the UI
   *
   * @param {PropWidgetDefinition} widget
   *
   * @returns this
   *
   * @example
   * setWidget(new PropWidgetDefinitionBuilder()
   * .setKind("text")
   * .build())
   */
  setWidget(widget: PropWidgetDefinition): this {
    if (widget.kind === "secret") {
      throw new Error(
        "Cannot create prop with secret widget. Use addSecretProp() to create those.",
      );
    }
    this.prop.widget = widget;
    return this;
  }
}

export interface SecretPropDefinition extends PropDefinition {
  hasInputSocket: boolean;
}

export interface ISecretPropBuilder {
  setName(name: string): this;

  setSecretKind(kind: string): this;

  setDocLinkRef(ref: string): this;

  setDocLink(link: string): this;

  addValidation(validation: Validation): this;

  skipInputSocket(): this;

  build(): SecretPropDefinition;
}

/**
 * Creates a prop [and a socket] in an asset with which to connect a secret
 *
 * @example
 *  const secretPropName = new SecretPropBuilder()
 *   .setName("credential")
 *   .setSecretKind("DigitalOcean Credential")
 *  .build();
 */
export class SecretPropBuilder implements ISecretPropBuilder {
  prop = <SecretPropDefinition>{};

  constructor() {
    this.prop = <SecretPropDefinition>{};
    this.prop.kind = "string";
    this.prop.widget = {
      kind: "secret",
      options: [],
    };
    this.prop.hasInputSocket = true;
  }

  /**
   * The secret prop name. This will appear in the model UI and can be any value
   *
   * @param {string} name - the name of the secret prop
   *
   * @returns this
   *
   * @example
   * .setName("token")
   */
  setName(name: string): this {
    this.prop.name = name;
    return this;
  }

  /**
   * The type of the secret - relates to the Secret Definition Name
   *
   *
   * @returns this
   *
   * @example
   * .setSecretKind("DigitalOcean Credential")
   * @param kind {string}
   */
  setSecretKind(kind: string): this {
    this.prop.widget?.options.push({ label: "secretKind", value: kind });
    return this;
  }

  setDocLinkRef(ref: string): this {
    this.prop.docLinkRef = ref;
    return this;
  }

  setDocLink(link: string): this {
    this.prop.docLink = link;
    return this;
  }

  addValidation(validation: Validation): this {
    if (!this.prop.validations) {
      this.prop.validations = [];
    }
    this.prop.validations.push(validation);
    return this;
  }

  /**
   * Whether the prop should disable the auto-creation of an input socket
   *
   * @returns this
   *
   * @example
   *  .skipInputSocket()
   */
  skipInputSocket(): this {
    this.prop.hasInputSocket = false;
    return this;
  }

  build(): SecretPropDefinition {
    if (
      this.prop.widget?.options?.find(
        (option) => option.label === "secretKind",
      ) === undefined
    ) {
      throw new Error("must call setSecretKind() before build()");
    }

    return this.prop;
  }
}

export interface SecretDefinition {
  name: string;
  props: PropDefinition[];
}

export interface ISecretDefinitionBuilder {
  addProp(prop: PropDefinition): this;

  setName(name: string): this;

  build(): SecretDefinition;
}

/**
 * Creates a secret to be used with a set of assets
 *
 * @example
 * const secretDefinition = new SecretDefinitionBuilder()
 *          .setName("DigitalOcean Token")
 *         .addProp(
 *             new PropBuilder()
 *             .setKind("string")
 *             .setName("token")
 *             .setWidget(
 *                 new PropWidgetDefinitionBuilder()
 *                 .setKind("password")
 *                 .build()
 *             )
 *             .build()
 *         )
 *         .build();
 */
export class SecretDefinitionBuilder implements ISecretDefinitionBuilder {
  definition: SecretDefinition;

  constructor() {
    this.definition = <SecretDefinition>{};
    this.definition.name = "";
    this.definition.props = [];
  }

  /**
   * The secret name. This corresponds to the kind of secret
   *
   * @param {string} name - the name of the secret kind
   *
   * @returns this
   *
   * @example
   * .setName("DigitalOcean Token")
   */
  setName(name: string): this {
    this.definition.name = name;
    return this;
  }

  /**
   * Adds a Prop to the secret definition. These define the form fields for the secret input
   *
   * @param prop {PropDefinition}
   *
   * @returns this
   *
   * @example
   *   .addProp(new PropBuilder()
   *     .setName("token")
   *     .setKind("string")
   *     .setWidget(new PropWidgetDefinitionBuilder().setKind("password").build())
   *     .build())
   */
  addProp(prop: PropDefinition): this {
    this.definition.props?.push(prop);
    return this;
  }

  build(): SecretDefinition {
    const def = this.definition;

    if (def.name.length === 0) {
      throw new Error("Cannot build SecretDefinition with empty name");
    }

    if (def.props.length === 0) {
      throw new Error("Cannot build SecretDefinition with no props");
    }

    return this.definition;
  }
}

export interface Asset {
  props: PropDefinition[];
  secretProps: SecretPropDefinition[];
  secretDefinition?: PropDefinition[];
  resourceProps: PropDefinition[];
  siPropValueFroms: SiPropValueFromDefinition[];
  inputSockets: SocketDefinition[];
  outputSockets: SocketDefinition[];
  docLinks: Record<string, string>;
}

export interface IAssetBuilder {
  addProp(prop: PropDefinition): this;

  addSecretProp(prop: SecretPropDefinition): this;

  defineSecret(definition: SecretDefinition): this;

  addResourceProp(prop: PropDefinition): this;

  addInputSocket(socket: SocketDefinition): this;

  addOutputSocket(socket: SocketDefinition): this;

  addSiPropValueFrom(siPropValueFrom: SiPropValueFromDefinition): this;

  addDocLink(key: string, value: string): this;

  build(): Asset;
}

export class AssetBuilder implements IAssetBuilder {
  asset = <Asset>{};

  constructor() {
    this.asset = <Asset>{};
  }

  addProp(prop: PropDefinition) {
    if (!this.asset.props) {
      this.asset.props = [];
    }
    this.asset.props?.push(prop);
    return this;
  }

  addSecretProp(prop: SecretPropDefinition) {
    if (!this.asset.secretProps) {
      this.asset.secretProps = [];
    }

    if (prop.hasInputSocket) {
      const secretKind = prop.widget?.options?.find(
        (option) => option.label === "secretKind",
      )?.value;

      if (secretKind === undefined) {
        throw new Error(`Could not find secretKind for ${prop.name}`);
      }

      this.addInputSocket(
        new SocketDefinitionBuilder()
          .setArity("one")
          .setName(secretKind)
          .build(),
      );

      prop.valueFrom = new ValueFromBuilder()
        .setKind("inputSocket")
        .setSocketName(secretKind)
        .build();
    }

    this.asset.secretProps?.push(prop);

    return this;
  }

  defineSecret(definition: SecretDefinition): this {
    this.asset.secretDefinition = definition.props;
    this.addSecretProp(
      new SecretPropBuilder()
        .setName(definition.name)
        .setSecretKind(definition.name)
        .skipInputSocket()
        .build(),
    );

    this.addOutputSocket(
      new SocketDefinitionBuilder()
        .setArity("one")
        .setName(definition.name)
        .setValueFrom(
          new ValueFromBuilder()
            .setKind("prop")
            .setPropPath(["root", "secrets", definition.name])
            .build(),
        )
        .build(),
    );

    return this;
  }

  addResourceProp(prop: PropDefinition) {
    if (!this.asset.resourceProps) {
      this.asset.resourceProps = [];
    }
    this.asset.resourceProps?.push(prop);
    return this;
  }

  addInputSocket(socket: SocketDefinition) {
    if (!this.asset.inputSockets) {
      this.asset.inputSockets = [];
    }
    this.asset.inputSockets?.push(socket);
    return this;
  }

  addOutputSocket(socket: SocketDefinition) {
    if (!this.asset.outputSockets) {
      this.asset.outputSockets = [];
    }
    this.asset.outputSockets?.push(socket);
    return this;
  }

  addSiPropValueFrom(siPropValueFrom: SiPropValueFromDefinition): this {
    if (!this.asset.siPropValueFroms) {
      this.asset.siPropValueFroms = [];
    }
    this.asset.siPropValueFroms.push(siPropValueFrom);
    return this;
  }

  addDocLink(key: string, value: string) {
    if (!this.asset.docLinks) {
      this.asset.docLinks = {};
    }
    this.asset.docLinks[key] = value;
    return this;
  }

  build() {
    if (this.asset.secretDefinition && this.asset.secretProps?.length !== 1) {
      throw new Error(
        "Secret defining schema shouldn't define any extra secret props",
      );
    }

    return this.asset;
  }
}
