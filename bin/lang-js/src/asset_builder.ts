import { parseConnectionAnnotation } from "@scope/ts-lib-deno";
import Joi from "npm:joi";

export type ValueFromKind = "inputSocket" | "outputSocket" | "prop";

export interface ValueFrom {
  kind: ValueFromKind;
  /** @deprecated Sockets are no longer supported and will be removed. */
  socket_name?: string;
  prop_path?: string[];
}

export interface IValueFromBuilder {
  setKind(kind: ValueFromKind): this;

  /** @deprecated Sockets are no longer supported and will be removed. */
  setSocketName(name: string): this;

  setPropPath(path: string[]): this;

  build(): ValueFrom;
}

/**
 * Gets a value from a prop
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
   * @param kind {string} [prop]
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
   * @deprecated Sockets are no longer supported and will be removed.
   *
   * @example
   * .setSocketName("Region")
   */
  setSocketName(name: string): this {
    if (
      this.valueFrom.kind !== "inputSocket" &&
      this.valueFrom.kind !== "outputSocket"
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

/** @deprecated Sockets are no longer supported and will be removed. */
export type SocketDefinitionArityType = "many" | "one";

/** @deprecated Sockets are no longer supported and will be removed. */
export interface SocketDefinition {
  name: string;
  arity: SocketDefinitionArityType;
  connectionAnnotations: string;
  uiHidden?: boolean;
  valueFrom?: ValueFrom;
}

/** @deprecated Sockets are no longer supported and will be removed. */
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
 * @deprecated Sockets are no longer supported and will be removed.
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
   * @deprecated Sockets are no longer supported and will be removed.
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
      this.connectionAnnotations.map((a) => a.toLowerCase().trim())
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
   * @deprecated Sockets are no longer supported and will be removed.
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
   * @deprecated Sockets are no longer supported and will be removed.
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
   * @deprecated Sockets are no longer supported and will be removed.
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
   * @deprecated Sockets are no longer supported and will be removed.
   *
   * @example
   *  .setName("Subnet ID")
   */
  setUiHidden(hidden: boolean): this {
    this.socket.uiHidden = hidden;
    return this;
  }

  /**
   * DEPRECATED: this method no longer does anything. It will be ignored
   * when executing the asset function. Please use the asset editing
   * interface to perform equivalent functionality.
   *
   * @param {ValueFrom} valueFrom
   *
   * @returns this
   *
   * @deprecated Sockets are no longer supported and will be removed.
   *
   * @example
   *  .setValueFrom(new ValueFromBuilder()
   *    .setKind("prop")
   *    .setPropPath(["root", "si", "name"])
   *    .build())
   */
  setValueFrom(valueFrom: ValueFrom): this {
    this.socket.valueFrom = valueFrom;
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

  setCreateOnly(): this;

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
  implements IPropWidgetDefinitionBuilder
{
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
   * Set this prop as create only prop. This means that when
   * the component has a resource attached, it will be marked
   * as uneditable in the Attributes panel
   */
  setCreateOnly(): this {
    if (!this.propWidget.options) {
      this.propWidget.options = [];
    }

    this.propWidget.options.push(<Option>{
      label: "si_create_only_prop",
      value: "true",
    });

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
   * DEPRECATED: this method no longer does anything. It will be ignored
   * when executing the asset function. Please use the asset editing
   * interface to perform equivalent functionality.
   *
   * If the entry is new, you will need to regenerate the asset first!
   *
   * In the past, this was used to set the value of this entry using a
   * ValueFromBuilder.
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
  implements ISiPropValueFromDefinitionBuilder
{
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
  | "float"
  | "integer"
  | "json"
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
  validationFormat?: string; // A JSON.stringify()-ed Joi.Descriptor
  suggestSources?: PropSuggestion[];
  suggestAsSourceFor?: PropSuggestion[];
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

  setValidationFormat(format: Joi.Schema): this;

  suggestSource(suggestion: PropSuggestion): this;

  suggestAsSourceFor(suggestion: PropSuggestion): this;

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
        "setEntry can only be called on prop that are arrays or maps"
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
   * @param kind {PropDefinitionKind} [array | boolean | float | integer | json | map | object | string]
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
   * DEPRECATED: this method no longer does anything. It will be ignored
   * when executing the asset function. Please use the asset editing
   * interface to perform equivalent functionality.
   *
   * If the prop is new, you will need to regenerate the asset first!
   *
   * In the past, this was used to set the value of this prop using a
   * ValueFromBuilder.
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
        "Cannot create prop with secret widget. Use addSecretProp() to create those."
      );
    }
    this.prop.widget = widget;
    return this;
  }

  /**
   * Suggests another prop (on another component) that this prop typically gets its value from.
   * The UI will use this to search for other components that could provide the value and
   * suggest them to the user when they edit the prop.
   *
   * @param {PropSuggestion} suggestion - the schema and prop path the UI should suggest to the user.
   *
   * @returns this
   *
   * @example
   * .suggestSource({ schema: "AWS::EC2::VPC", prop: "/resource_value/VpcId" })
   */
  suggestSource(suggestion: PropSuggestion) {
    this.prop.suggestSources ??= [];
    this.prop.suggestSources.push(suggestion);
    return this;
  }

  /**
   * Suggests another prop (on another component) that typically gets its value from this prop.
   * The UI will use this to search for other components that could provide the value and
   * suggest them to the user when they edit the prop.
   *
   * @param {PropSuggestion} suggestion - the schema and prop path the UI should suggest to the user.
   *
   * @returns this
   *
   * @example
   * .suggestAsSourceFor({ schema: "AWS::EC2::Subnet", prop: "/domain/VpcId" })
   */
  suggestAsSourceFor(suggestion: PropSuggestion) {
    this.prop.suggestAsSourceFor ??= [];
    this.prop.suggestAsSourceFor.push(suggestion);
    return this;
  }
}

/**
 * Suggestion for a prop that can be connected to
 *
 * @see PropBuilder.suggestSource()
 * @see PropBuilder.suggestAsSourceFor()
 */
export interface PropSuggestion {
  /**
   * The schema this prop exists on
   *
   * @example "AWS::EC2::VPC"
   */
  schema: string;

  /**
   * The path to the prop within the schema
   *
   * @example "/resource_value/VpcId"
   */
  prop: string;
}

export interface SecretPropDefinition extends PropDefinition {
  /** @deprecated Sockets are no longer supported and will be removed. */
  hasInputSocket: boolean;
  /** @deprecated Sockets are no longer supported and will be removed. */
  connectionAnnotation: string;
}

export interface ISecretPropBuilder {
  setName(name: string): this;

  setSecretKind(kind: string): this;

  /** @deprecated Sockets are no longer supported and will be removed. */
  setConnectionAnnotation(annotation: string): this;

  setDocLinkRef(ref: string): this;

  setDocLink(link: string): this;

  /** @deprecated Sockets are no longer supported and will be removed. */
  skipInputSocket(): this;

  suggestSource(suggestion: PropSuggestion): this;

  suggestAsSourceFor(suggestion: PropSuggestion): this;

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

  /** @deprecated Sockets are no longer supported and will be removed. */
  setConnectionAnnotation(annotation: string): this {
    this.prop.connectionAnnotation = annotation;
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

  /**
   * Suggests another prop (on another component) that this prop typically gets its value from.
   * The UI will use this to search for other components that could provide the value and
   * suggest them to the user when they edit the prop.
   *
   * @param {PropSuggestion} suggestion - the schema and prop path the UI should suggest to the user.
   *
   * @returns this
   *
   * @example
   * .suggestSource({ schema: "AWS::EC2::VPC", prop: "/resource_value/VpcId" })
   */
  suggestSource(suggestion: PropSuggestion) {
    this.prop.suggestSources ??= [];
    this.prop.suggestSources.push(suggestion);
    return this;
  }

  /**
   * Suggests another prop (on another component) that typically gets its value from this prop.
   * The UI will use this to search for other components that could provide the value and
   * suggest them to the user when they edit the prop.
   *
   * @param {PropSuggestion} suggestion - the schema and prop path the UI should suggest to the user.
   *
   * @returns this
   *
   * @example
   * .suggestAsSourceFor({ schema: "AWS::EC2::Subnet", prop: "/domain/VpcId" })
   */
  suggestAsSourceFor(suggestion: PropSuggestion) {
    this.prop.suggestAsSourceFor ??= [];
    this.prop.suggestAsSourceFor.push(suggestion);
    return this;
  }

  /**
   * Whether the prop should disable the auto-creation of an input socket
   *
   * @returns this
   *
   * @deprecated Sockets are no longer supported and will be removed.
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
        (option) => option.label === "secretKind"
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
  /** @deprecated Sockets are no longer supported and will be removed. */
  connectionAnnotations?: string;
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
    this.definition.connectionAnnotations = "";
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

  /**
   * Adds the specified connection annotations to the output socket for the secret
   *
   * @param {string} connectionAnnotations - the connection annotations to create for the output socket.
   *
   * @returns this
   *
   * @deprecated Sockets are no longer supported and will be removed.
   *
   * @example
   * .setConnectionAnnotation("Registry Token")
   */
  setConnectionAnnotation(connectionAnnotations: string): this {
    this.definition.connectionAnnotations = connectionAnnotations;
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
  /** @deprecated Sockets are no longer supported and will be removed. */
  inputSockets: SocketDefinition[];
  /** @deprecated Sockets are no longer supported and will be removed. */
  outputSockets: SocketDefinition[];
  docLinks: Record<string, string>;
}

export interface IAssetBuilder {
  addProp(prop: PropDefinition): this;

  addSecretProp(prop: SecretPropDefinition): this;

  defineSecret(definition: SecretDefinition): this;

  addResourceProp(prop: PropDefinition): this;

  /** @deprecated Sockets are no longer supported and will be removed. */
  addInputSocket(socket: SocketDefinition): this;

  /** @deprecated Sockets are no longer supported and will be removed. */
  addOutputSocket(socket: SocketDefinition): this;

  addSiPropValueFrom(siPropValueFrom: SiPropValueFromDefinition): this;

  addDocLink(key: string, value: string): this;

  build(): Asset;
}

/**
 * Represents a builder for creating System Initiative Asset Schemas.
 *
 * @example
 * const asset = new AssetBuilder();
 *
 * const myProp = new PropBuilder().setName("myProp").setKind("string").build();
 * asset.addProp(myProp);
 *
 * return asset.build();
 */
export class AssetBuilder implements IAssetBuilder {
  asset = <Asset>{};

  constructor() {
    this.asset = <Asset>{};
  }

  /**
   * Adds a prop to the asset.
   *
   * @param prop - The prop definition to add
   * @returns This AssetBuilder instance for method chaining
   */
  addProp(prop: PropDefinition) {
    if (!this.asset.props) {
      this.asset.props = [];
    }
    this.asset.props?.push(prop);
    return this;
  }

  /**
   * Adds a secret prop to the asset.
   *
   * @param prop - The secret prop definition to add
   * @returns This AssetBuilder instance for method chaining
   */
  addSecretProp(prop: SecretPropDefinition) {
    if (!this.asset.secretProps) {
      this.asset.secretProps = [];
    }

    if (prop.hasInputSocket) {
      const secretKind = prop.widget?.options?.find(
        (option) => option.label === "secretKind"
      )?.value;

      if (secretKind === undefined) {
        throw new Error(`Could not find secretKind for ${prop.name}`);
      }

      this.addInputSocket(
        new SocketDefinitionBuilder()
          .setArity("one")
          .setName(secretKind)
          .build()
      );

      prop.valueFrom = new ValueFromBuilder()
        .setKind("inputSocket")
        .setSocketName(secretKind)
        .build();
    }

    this.asset.secretProps?.push(prop);

    return this;
  }

  /**
   * Adds a secret to the asset.
   *
   * @param definition - The secret definition to add
   * @returns This AssetBuilder instance for method chaining
   */
  defineSecret(definition: SecretDefinition): this {
    this.asset.secretDefinition = definition.props;
    this.addSecretProp(
      new SecretPropBuilder()
        .setName(definition.name)
        .setSecretKind(definition.name)
        .skipInputSocket()
        .build()
    );

    const outputSocketBuilder = new SocketDefinitionBuilder()
      .setArity("one")
      .setName(definition.name)
      .setValueFrom(
        new ValueFromBuilder()
          .setKind("prop")
          .setPropPath(["root", "secrets", definition.name])
          .build()
      );

    if (
      definition.connectionAnnotations &&
      definition.connectionAnnotations !== ""
    ) {
      outputSocketBuilder.setConnectionAnnotation(
        definition.connectionAnnotations
      );
    }

    const outputSocket = outputSocketBuilder.build();
    this.addOutputSocket(outputSocket);

    return this;
  }

  /**
   * Adds a resource prop to the asset.
   *
   * @param prop - The prop definition to add
   * @returns This AssetBuilder instance for method chaining
   */
  addResourceProp(prop: PropDefinition) {
    if (!this.asset.resourceProps) {
      this.asset.resourceProps = [];
    }
    this.asset.resourceProps?.push(prop);
    return this;
  }

  /**
   * Adds an input socket to the asset.
   *
   * @deprecated Sockets are no longer supported and will be removed.
   *
   * @param socket - The socket definition to add
   * @returns This AssetBuilder instance for method chaining
   */
  addInputSocket(socket: SocketDefinition) {
    if (!this.asset.inputSockets) {
      this.asset.inputSockets = [];
    }
    this.asset.inputSockets?.push(socket);
    return this;
  }

  /**
   * Adds an output socket to the asset.
   *
   * @deprecated Sockets are no longer supported and will be removed.
   *
   * @param socket - The socket definition to add
   * @returns This AssetBuilder instance for method chaining
   */
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

  /**
   * Adds a doc link to the asset.
   *
   * @param key - the name of the doc link
   * @param value - the value for the doc link
   * @returns This AssetBuilder instance for method chaining
   */
  addDocLink(key: string, value: string) {
    if (!this.asset.docLinks) {
      this.asset.docLinks = {};
    }
    this.asset.docLinks[key] = value;
    return this;
  }

  build() {
    if (this.asset.secretDefinition && this.asset.outputSockets?.length > 1) {
      throw new Error(
        "secret defining assets cannot have more than one output socket since it can only output the secret corresponding to the definition"
      );
    }
    return this.asset;
  }
}
