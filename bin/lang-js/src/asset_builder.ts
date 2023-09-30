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

export class ValueFromBuilder implements IValueFromBuilder {
    valueFrom = <ValueFrom>{};

    constructor() {
        this.valueFrom = <ValueFrom>{};
    }

    setKind(kind: ValueFromKind): this {
        this.valueFrom.kind = kind;
        return this;
    }

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

    setPropPath(path: string[]): this {
        if (this.valueFrom.kind !== "prop") {
            return this;
        }

        this.valueFrom.prop_path = path;
        return this;
    }

    build(): ValueFrom {
        return this.valueFrom;
    }
}

export type SocketDefinitionArityType = "many" | "one";
export interface SocketDefinition {
    name: string;
    arity: SocketDefinitionArityType;
    uiHidden?: boolean;
    valueFrom?: ValueFrom;
}

export interface ISocketDefinitionBuilder {
    setName(name: string): this;

    setArity(arity: SocketDefinitionArityType): this;

    setUiHidden(hidden: boolean): this;

    setValueFrom(valueFrom: ValueFrom): this;

    build(): SocketDefinition;
}

export class SocketDefinitionBuilder implements ISocketDefinitionBuilder {
    socket = <SocketDefinition>{};

    constructor() {
        this.socket = <SocketDefinition>{};
    }

    build(): SocketDefinition {
        return this.socket;
    }

    setArity(arity: SocketDefinitionArityType): this {
        this.socket.arity = arity;
        return this;
    }

    setName(name: string): this {
        this.socket.name = name;
        return this;
    }

    setUiHidden(hidden: boolean): this {
        this.socket.uiHidden = hidden;
        return this;
    }

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
            this.validation.kind !== "stringEquals" &&
            this.validation.kind !== "stringHasPrefix" &&
            this.validation.kind !== "stringInStringArray"
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
    | "color"
    | "comboBox"
    | "header"
    | "map"
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

export class PropWidgetDefinitionBuilder
    implements IPropWidgetDefinitionBuilder
{
    propWidget = <PropWidgetDefinition>{};

    constructor() {
        this.propWidget = <PropWidgetDefinition>{};
    }

    setKind(kind: PropWidgetDefinitionKind): this {
        this.propWidget.kind = kind;
        return this;
    }

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    addOption(key: string, value: string): this {
        if (!this.propWidget.options) {
            this.propWidget.options = [];
        }

        this.propWidget.options.push(<Option>{
            label: key,
            value: value,
        });
        return this;
    }

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

export class MapKeyFuncBuilder implements IMapKeyFuncBuilder {
    mapKeyFunc = <MapKeyFunc>{};

    constructor() {
        this.mapKeyFunc = <MapKeyFunc>{};
    }

    build(): MapKeyFunc {
        return this.mapKeyFunc;
    }

    setKey(key: string): this {
        this.mapKeyFunc.key = key;
        return this;
    }

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
    children?: PropDefinition[];
    entry?: PropDefinition;
    widget?: PropWidgetDefinition;
    valueFrom?: ValueFrom;
    hidden?: boolean;
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    defaultValue?: any;
    validations?: Validation[];
    mapKeyFuncs?: MapKeyFunc[];
}

export interface IPropBuilder {
    setName(name: string): this;

    setKind(kind: PropDefinitionKind): this;

    setDocLinkRef(ref: string): this;

    setDocLink(link: string): this;

    addChild(child: PropDefinition): this;

    setEntry(entry: PropDefinition): this;

    setWidget(widget: PropWidgetDefinition): this;

    setValueFrom(valueFrom: ValueFrom): this;

    setHidden(hidden: boolean): this;

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    setDefaultValue(value: any): this;

    addValidation(validation: Validation): this;

    addMapKeyFunc(func: MapKeyFunc): this;

    build(): PropDefinition;
}

export class PropBuilder implements IPropBuilder {
    prop = <PropDefinition>{};

    constructor() {
        this.prop = <PropDefinition>{};
    }

    addChild(child: PropDefinition): this {
        if (this.prop.kind !== "object") {
            throw new Error(
                "addChild can only be called on props that are objects"
            );
        }

        if (!this.prop.children) {
            this.prop.children = [];
        }

        this.prop.children.push(child);
        return this;
    }

    setEntry(entry: PropDefinition): this {
        if (this.prop.kind !== "array" && this.prop.kind !== "map") {
            throw new Error(
                "setEntry can only be called on prop that are arrays or maps"
            );
        }

        this.prop.entry = entry;
        return this;
    }

    addMapKeyFunc(func: MapKeyFunc): this {
        if (!this.prop.mapKeyFuncs) {
            this.prop.mapKeyFuncs = [];
        }
        this.prop.mapKeyFuncs.push(func);
        return this;
    }

    addValidation(validation: Validation): this {
        if (!this.prop.validations) {
            this.prop.validations = [];
        }
        this.prop.validations.push(validation);
        return this;
    }

    build(): PropDefinition {
        return this.prop;
    }

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    setDefaultValue(value: any): this {
        this.prop.defaultValue = value;
        return this;
    }

    setDocLink(link: string): this {
        this.prop.docLink = link;
        return this;
    }

    setDocLinkRef(ref: string): this {
        this.prop.docLinkRef = ref;
        return this;
    }

    setHidden(hidden: boolean): this {
        this.prop.hidden = hidden;
        return this;
    }

    setKind(kind: PropDefinitionKind): this {
        this.prop.kind = kind;
        return this;
    }

    setName(name: string): this {
        this.prop.name = name;
        return this;
    }

    setValueFrom(valueFrom: ValueFrom): this {
        this.prop.valueFrom = valueFrom;
        return this;
    }

    setWidget(widget: PropWidgetDefinition): this {
        if(widget.kind === 'secret') {
          throw new Error("Cannot create prop with secret widget. Use addSecretProp() to create those.");
        }
        this.prop.widget = widget;
        return this;
    }
}

// eslint-disable-next-line @typescript-eslint/no-empty-interface
export interface SecretPropDefinition extends PropDefinition {}

export interface ISecretPropBuilder {
    setName(name: string): this;

    setSecretKind(kind: string): this;

    setDocLinkRef(ref: string): this;

    setDocLink(link: string): this;

    addValidation(validation: Validation): this;

    build(): SecretPropDefinition;
}

export class SecretPropBuilder implements ISecretPropBuilder {
    prop = <SecretPropDefinition>{};

    constructor() {
        this.prop = <SecretPropDefinition>{};
        this.prop.kind = "string";
        this.prop.widget = {
            kind: "secret",
            options: [],
        };
    }
    setName(name: string): this {
        this.prop.name = name;
        return this;
    }

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
    props?: PropDefinition[];
}

export interface ISecretDefinitionBuilder {
    addProp(prop: PropDefinition): this;

    build(): SecretDefinition;
}

export class SecretDefinitionBuilder implements ISecretDefinitionBuilder {
    definition: SecretDefinition;

    constructor() {
        this.definition = <SecretDefinition>{};
    }

    setName(name: string): this {
        this.definition.name = name;
        return this;
    }

    addProp(prop: PropDefinition): this {
        if (!this.definition.props) {
            this.definition.props = [];
        }
        this.definition.props?.push(prop);
        return this;
    }

    build(): SecretDefinition {
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
        this.asset.secretProps?.push(prop);
        return this;
    }

    defineSecret(definition: SecretDefinition): this {
        this.asset.secretDefinition = definition.props;
        this.addSecretProp(
            new SecretPropBuilder()
                .setName(definition.name)
                .setSecretKind(definition.name)
                .build()
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
        return this.asset;
    }
}
