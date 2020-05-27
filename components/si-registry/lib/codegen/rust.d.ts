import { ObjectTypes, EntityObject } from "../systemComponent";
import * as PropPrelude from "../components/prelude";
import { Props, IntegrationService } from "../attrList";
interface RustTypeAsPropOptions {
    reference?: boolean;
    option?: boolean;
}
interface AgentIntegrationService {
    agentName: string;
    entity: EntityObject;
    integrationName: string;
    integrationServiceName: string;
}
interface PropertyUpdate {
    from: PropPrelude.Props;
    to: PropPrelude.Props;
}
interface PropertyEitherSet {
    entries: PropPrelude.Props[];
}
export declare class RustFormatter {
    systemObject: ObjectTypes;
    constructor(systemObject: RustFormatter["systemObject"]);
    entityActionMethodNames(): string[];
    hasCreateMethod(): boolean;
    hasEditEithersForAction(propAction: PropPrelude.PropAction): boolean;
    hasEditUpdatesForAction(propAction: PropPrelude.PropAction): boolean;
    hasEditUpdatesAndEithers(): boolean;
    isComponentObject(): boolean;
    isEntityActionMethod(propMethod: PropPrelude.PropMethod): boolean;
    isEntityEditMethod(propMethod: PropPrelude.PropMethod): boolean;
    isEntityEventObject(): boolean;
    isEntityObject(): boolean;
    isMigrateable(): boolean;
    isStorable(): boolean;
    actionProps(): PropPrelude.PropAction[];
    componentName(): string;
    componentConstraintsName(): string;
    entityEditMethodName(propMethod: PropPrelude.PropMethod): string;
    entityEditMethods(): PropPrelude.PropAction[];
    entityEditProperty(propAction: PropPrelude.PropAction): Props;
    entityEditPropertyField(propAction: PropPrelude.PropAction): string;
    entityEditPropertyType(propAction: PropPrelude.PropAction): string;
    entityEditPropertyUpdates(propAction: PropPrelude.PropAction): PropertyUpdate[];
    allEntityEditPropertyUpdates(): PropertyUpdate[];
    entityEditPropertyEithers(): PropertyEitherSet[];
    entityEditPropertyUpdateMethodName(propertyUpdate: PropertyUpdate): string;
    entityEventName(): string;
    entityName(): string;
    entityPropertiesName(): string;
    errorType(): string;
    modelName(): string;
    modelServiceMethodName(propMethod: PropPrelude.PropMethod | PropPrelude.PropAction): string;
    structName(): string;
    typeName(): string;
    implTryFromForPropertyUpdate(propertyUpdate: PropertyUpdate): string;
    implListRequestType(renderOptions?: RustTypeAsPropOptions): string;
    implListReplyType(renderOptions?: RustTypeAsPropOptions): string;
    implServiceRequestType(propMethod: PropPrelude.PropMethod, renderOptions?: RustTypeAsPropOptions): string;
    implServiceReplyType(propMethod: PropPrelude.PropMethod, renderOptions?: RustTypeAsPropOptions): string;
    implServiceTraceName(propMethod: PropPrelude.PropMethod | PropPrelude.PropAction): string;
    implServiceMethodName(propMethod: PropPrelude.PropMethod | PropPrelude.PropAction): string;
    implServiceEntityAction(propMethod: PropPrelude.PropMethod): string;
    implServiceEntityEdit(propMethod: PropPrelude.PropMethod): string;
    implServiceCommonCreate(propMethod: PropPrelude.PropMethod): string;
    implServiceEntityCreate(propMethod: PropPrelude.PropMethod): string;
    implServiceGet(propMethod: PropPrelude.PropMethod): string;
    implServiceList(propMethod: PropPrelude.PropMethod): string;
    implServiceComponentPick(propMethod: PropPrelude.PropMethod): string;
    implServiceCustomMethod(propMethod: PropPrelude.PropMethod): string;
    implServiceAuth(propMethod: PropPrelude.PropMethod): string;
    implServiceAuthCall(propMethod: PropPrelude.PropMethod): string;
    serviceMethods(): string;
    rustFieldNameForProp(prop: Props): string;
    rustTypeForProp(prop: Props, renderOptions?: RustTypeAsPropOptions): string;
    implCreateNewArgs(): string;
    implCreatePassNewArgs(): string;
    implServiceMethodListResultToReply(): string;
    implServiceMethodCreateDestructure(): string;
    naturalKey(): string;
    implCreateSetProperties(): string;
    implCreateAddToTenancy(): string;
    storableIsMvcc(): string;
    storableValidateFunction(): string;
    storableOrderByFieldsByProp(topProp: PropPrelude.PropObject, prefix: string): string;
    storableOrderByFieldsFunction(): string;
    storableReferentialFieldsFunction(): string;
}
export declare class RustFormatterService {
    serviceName: string;
    systemObjects: ObjectTypes[];
    constructor(serviceName: string);
    systemObjectsAsFormatters(): RustFormatter[];
    implServiceStructBody(): string;
    implServiceNewConstructorArgs(): string;
    implServiceStructConstructorReturn(): string;
    implServiceTraitName(): string;
    implServerName(): string;
    implServiceMigrate(): string;
    hasEntities(): boolean;
    isMigrateable(prop: ObjectTypes): boolean;
    hasMigratables(): boolean;
}
export declare class RustFormatterAgent {
    agentName: string;
    entity: EntityObject;
    entityFormatter: RustFormatter;
    integrationName: string;
    integrationServiceName: string;
    serviceName: string;
    systemObjects: ObjectTypes[];
    constructor(serviceName: string, agent: AgentIntegrationService);
    systemObjectsAsFormatters(): RustFormatter[];
    actionProps(): PropPrelude.PropAction[];
    entityActionMethodNames(): string[];
    dispatcherBaseTypeName(): string;
    dispatcherTypeName(): string;
    dispatchFunctionTraitName(): string;
}
export declare class CodegenRust {
    serviceName: string;
    constructor(serviceName: string);
    hasModels(): boolean;
    hasServiceMethods(): boolean;
    hasEntityIntegrationServcices(): boolean;
    entities(): EntityObject[];
    entityActions(entity: EntityObject): PropPrelude.PropAction[];
    entityintegrationServicesFor(entity: EntityObject): IntegrationService[];
    entityIntegrationServices(): AgentIntegrationService[];
    generateGenMod(): Promise<void>;
    generateGenModelMod(): Promise<void>;
    generateGenService(): Promise<void>;
    generateGenModel(systemObject: ObjectTypes): Promise<void>;
    generateGenAgentMod(): Promise<void>;
    generateGenAgent(agent: AgentIntegrationService): Promise<void>;
    formatCode(): Promise<void>;
    writeCode(filename: string, code: string): Promise<void>;
}
export {};
