export interface ValueFrom {
    type: string;
    socketName?: string;
    propPath?: string[];
}

export interface IValueFromBuilder {
    setType(type: string): this;

    setSocketName(name: string): this;

    setPropPath(path: string[]): this;

    build(): ValueFrom;
}

export class ValueFromBuilder implements IValueFromBuilder {
    valueFrom = <ValueFrom>{};

    constructor() {
        this.valueFrom = <ValueFrom>{};
    }

    setType(type: string): this {
        this.valueFrom.type = type;
        return this;
    }

    setSocketName(name: string): this {
        if (
            this.valueFrom.type !== "inputSocket" &&
            this.valueFrom.type !== "outputSocket"
        ) {
            return this;
        }

        this.valueFrom.socketName = name;
        return this;
    }

    setPropPath(path: string[]): this {
        if (this.valueFrom.type !== "prop") {
            return this;
        }

        this.valueFrom.propPath = path;
        return this;
    }

    build(): ValueFrom {
        return this.valueFrom;
    }
}

export type SocketDefinitionType = "many" | "one";

export interface SocketDefinition {
    name: string;
    arity: SocketDefinitionType;
    uiHidden?: boolean;
    valueFrom?: ValueFrom;
}

export interface ISocketDefinitionBuilder {
    setName(name: string): this;

    setArity(arity: SocketDefinitionType): this;

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

    setArity(arity: SocketDefinitionType): this {
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

export type ValidationType =
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
    type: ValidationType;
    funcUniqueId?: Record<string, unknown>;
    lowerBound?: number;
    upperBound?: number;
    expected?: string[];
    displayExpected?: boolean;
}

export interface IValidationBuilder {
    setType(type: ValidationType): this;

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
        if (this.validation.type !== "customValidation") {
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
        if (this.validation.type !== "stringInStringArray") {
            return this;
        }

        this.validation.displayExpected = display;
        return this;
    }

    addExpected(expected: string): this {
        if (
            this.validation.type !== "stringEquals" &&
            this.validation.type !== "stringHasPrefix" &&
            this.validation.type !== "stringInStringArray"
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
        if (this.validation.type !== "integerIsBetweenTwoIntegers") {
            return this;
        }
        this.validation.lowerBound = value;
        return this;
    }

    setType(type: ValidationType): this {
        this.validation.type = type;
        return this;
    }

    setUpperBound(value: number): this {
        if (this.validation.type !== "integerIsBetweenTwoIntegers") {
            return this;
        }
        this.validation.upperBound = value;
        return this;
    }
}

export type PropWidgetDefinitionKind =
    | "array"
    | "checkBox"
    | "color"
    | "comboBox"
    | "header"
    | "map"
    | "secretSelect"
    | "select"
    | "text"
    | "textArea";

export interface PropWidgetDefinition {
    kind: PropWidgetDefinitionKind;
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    options: Record<string, any>;
}

export interface IPropWidgetDefinitionBuilder {
    setKind(kind: string): this;

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    addOption(key: string, value: any): this;

    build(): PropWidgetDefinition;
}

export class PropWidgetDefinitionBuilder
    implements IPropWidgetDefinitionBuilder {
    propWidget = <PropWidgetDefinition>{};

    constructor() {
        this.propWidget = <PropWidgetDefinition>{};
    }

    setKind(kind: PropWidgetDefinitionKind): this {
        this.propWidget.kind = kind;
        return this;
    }

    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    addOption(key: string, value: any): this {
        if (!this.propWidget.options){
            this.propWidget.options = {};
        }

        this.propWidget.options[key] = value;
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
    implements ISiPropValueFromDefinitionBuilder {
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
    entry?: PropDefinition[];
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

    addEntry(entry: PropDefinition): this;

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
            return this;
        }

        if (!this.prop.children) {
            this.prop.children = [];
        }

        this.prop.children.push(child);
        return this;
    }

    addEntry(entry: PropDefinition): this {
        if (this.prop.kind !== "array") {
            return this;
        }

        if (!this.prop.entry) {
            this.prop.entry = [];
        }

        this.prop.entry.push(entry);
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
        this.prop.widget = widget;
        return this;
    }
}

export interface Asset {
    props: PropDefinition[];
    resourceProps: PropDefinition[];
    siPropValueFroms: SiPropValueFromDefinition[];
    inputSockets: SocketDefinition[];
    outputSockets: SocketDefinition[];
    docLinks: Record<string, string>;
}

export interface IAssetBuilder {
    addProp(prop: PropDefinition): this;

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
