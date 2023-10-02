type ValueFromKind = "inputSocket" | "outputSocket" | "prop";
interface ValueFrom {
    kind: ValueFromKind;
    socket_name?: string;
    prop_path?: string[];
}
interface IValueFromBuilder {
    setKind(kind: ValueFromKind): this;
    setSocketName(name: string): this;
    setPropPath(path: string[]): this;
    build(): ValueFrom;
}
declare class ValueFromBuilder implements IValueFromBuilder {
    valueFrom: ValueFrom;
    constructor();
    setKind(kind: ValueFromKind): this;
    setSocketName(name: string): this;
    setPropPath(path: string[]): this;
    build(): ValueFrom;
}
type SocketDefinitionArityType = "many" | "one";
interface SocketDefinition {
    name: string;
    arity: SocketDefinitionArityType;
    uiHidden?: boolean;
    valueFrom?: ValueFrom;
}
interface ISocketDefinitionBuilder {
    setName(name: string): this;
    setArity(arity: SocketDefinitionArityType): this;
    setUiHidden(hidden: boolean): this;
    setValueFrom(valueFrom: ValueFrom): this;
    build(): SocketDefinition;
}
declare class SocketDefinitionBuilder implements ISocketDefinitionBuilder {
    socket: SocketDefinition;
    constructor();
    build(): SocketDefinition;
    setArity(arity: SocketDefinitionArityType): this;
    setName(name: string): this;
    setUiHidden(hidden: boolean): this;
    setValueFrom(valueFrom: ValueFrom): this;
}
type ValidationKind = "customValidation" | "integerIsBetweenTwoIntegers" | "integerIsNotEmpty" | "stringEquals" | "stringHasPrefix" | "stringInStringArray" | "stringIsHexColor" | "stringIsNotEmpty" | "stringIsValidIpAddr";
interface Validation {
    kind: ValidationKind;
    funcUniqueId?: Record<string, unknown>;
    lowerBound?: number;
    upperBound?: number;
    expected?: string[];
    displayExpected?: boolean;
}
interface IValidationBuilder {
    setKind(kind: ValidationKind): this;
    addFuncUniqueId(key: string, value: unknown): this;
    setLowerBound(value: number): this;
    setUpperBound(value: number): this;
    addExpected(expected: string): this;
    setDisplayExpected(display: boolean): this;
    build(): Validation;
}
declare class ValidationBuilder implements IValidationBuilder {
    validation: Validation;
    constructor();
    addFuncUniqueId(key: string, value: unknown): this;
    build(): Validation;
    setDisplayExpected(display: boolean): this;
    addExpected(expected: string): this;
    setLowerBound(value: number): this;
    setKind(kind: ValidationKind): this;
    setUpperBound(value: number): this;
}
type PropWidgetDefinitionKind = "array" | "checkbox" | "color" | "comboBox" | "header" | "map" | "secret" | "select" | "text" | "textArea";
interface Option {
    label: string;
    value: string;
}
interface PropWidgetDefinition {
    kind: PropWidgetDefinitionKind;
    options: Option[];
}
interface IPropWidgetDefinitionBuilder {
    setKind(kind: string): this;
    addOption(key: string, value: string): this;
    build(): PropWidgetDefinition;
}
declare class PropWidgetDefinitionBuilder implements IPropWidgetDefinitionBuilder {
    propWidget: PropWidgetDefinition;
    constructor();
    setKind(kind: PropWidgetDefinitionKind): this;
    addOption(key: string, value: string): this;
    build(): PropWidgetDefinition;
}
interface MapKeyFunc {
    key: string;
    valueFrom?: ValueFrom;
}
interface IMapKeyFuncBuilder {
    setKey(key: string): this;
    setValueFrom(valueFrom: ValueFrom): this;
    build(): MapKeyFunc;
}
declare class MapKeyFuncBuilder implements IMapKeyFuncBuilder {
    mapKeyFunc: MapKeyFunc;
    constructor();
    build(): MapKeyFunc;
    setKey(key: string): this;
    setValueFrom(valueFrom: ValueFrom): this;
}
type SiPropValueFromDefinitionKind = "color" | "name" | "resourcePayload";
interface SiPropValueFromDefinition {
    kind: SiPropValueFromDefinitionKind;
    valueFrom: ValueFrom;
}
interface ISiPropValueFromDefinitionBuilder {
    setKind(kind: SiPropValueFromDefinitionKind): this;
    setValueFrom(valueFrom: ValueFrom): this;
    build(): SiPropValueFromDefinition;
}
declare class SiPropValueFromDefinitionBuilder implements ISiPropValueFromDefinitionBuilder {
    definition: SiPropValueFromDefinition;
    constructor();
    build(): SiPropValueFromDefinition;
    setKind(kind: SiPropValueFromDefinitionKind): this;
    setValueFrom(valueFrom: ValueFrom): this;
}
type PropDefinitionKind = "array" | "boolean" | "integer" | "map" | "object" | "string";
interface PropDefinition {
    name: string;
    kind: PropDefinitionKind;
    docLinkRef?: string;
    docLink?: string;
    children?: PropDefinition[];
    entry?: PropDefinition;
    widget?: PropWidgetDefinition;
    valueFrom?: ValueFrom;
    hidden?: boolean;
    defaultValue?: any;
    validations?: Validation[];
    mapKeyFuncs?: MapKeyFunc[];
}
interface IPropBuilder {
    setName(name: string): this;
    setKind(kind: PropDefinitionKind): this;
    setDocLinkRef(ref: string): this;
    setDocLink(link: string): this;
    addChild(child: PropDefinition): this;
    setEntry(entry: PropDefinition): this;
    setWidget(widget: PropWidgetDefinition): this;
    setValueFrom(valueFrom: ValueFrom): this;
    setHidden(hidden: boolean): this;
    setDefaultValue(value: any): this;
    addValidation(validation: Validation): this;
    addMapKeyFunc(func: MapKeyFunc): this;
    build(): PropDefinition;
}
declare class PropBuilder implements IPropBuilder {
    prop: PropDefinition;
    constructor();
    addChild(child: PropDefinition): this;
    setEntry(entry: PropDefinition): this;
    addMapKeyFunc(func: MapKeyFunc): this;
    addValidation(validation: Validation): this;
    build(): PropDefinition;
    setDefaultValue(value: any): this;
    setDocLink(link: string): this;
    setDocLinkRef(ref: string): this;
    setHidden(hidden: boolean): this;
    /**
    * The type of the prop
    *
    * @param {string} kind [array | boolean | integer | map | object | string]
    *
    * @returns this
    *
    * @example
    * .setKind("text")
    */
    setKind(kind: PropDefinitionKind): this;
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
    setName(name: string): this;
    setValueFrom(valueFrom: ValueFrom): this;
    setWidget(widget: PropWidgetDefinition): this;
}
interface SecretPropDefinition extends PropDefinition {
}
interface ISecretPropBuilder {
    setName(name: string): this;
    setSecretKind(kind: string): this;
    setDocLinkRef(ref: string): this;
    setDocLink(link: string): this;
    addValidation(validation: Validation): this;
    build(): SecretPropDefinition;
}
declare class SecretPropBuilder implements ISecretPropBuilder {
    prop: SecretPropDefinition;
    constructor();
    setName(name: string): this;
    setSecretKind(kind: string): this;
    setDocLinkRef(ref: string): this;
    setDocLink(link: string): this;
    addValidation(validation: Validation): this;
    build(): SecretPropDefinition;
}
interface SecretDefinition {
    name: string;
    props?: PropDefinition[];
}
interface ISecretDefinitionBuilder {
    addProp(prop: PropDefinition): this;
    build(): SecretDefinition;
}
declare class SecretDefinitionBuilder implements ISecretDefinitionBuilder {
    definition: SecretDefinition;
    constructor();
    setName(name: string): this;
    addProp(prop: PropDefinition): this;
    build(): SecretDefinition;
}
interface Asset {
    props: PropDefinition[];
    secretProps: SecretPropDefinition[];
    secretDefinition?: PropDefinition[];
    resourceProps: PropDefinition[];
    siPropValueFroms: SiPropValueFromDefinition[];
    inputSockets: SocketDefinition[];
    outputSockets: SocketDefinition[];
    docLinks: Record<string, string>;
}
interface IAssetBuilder {
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
declare class AssetBuilder implements IAssetBuilder {
    asset: Asset;
    constructor();
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
