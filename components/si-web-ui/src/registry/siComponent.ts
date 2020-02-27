import { DocumentNode } from "graphql";

export class SiComponent {
  typeName: string;
  name: string;
  icon: string;

  // Properties
  componentProperties: string[];

  // Hints
  hints: {
    constraintName: string;
    hintValue: string;
  }[];

  showActions: {
    displayName: string;
    mutation?: DocumentNode;
  }[];

  showProperties: {
    displayName: string;
    property: string;
    showAs: "text" | "textarea";
  }[];

  listHeaders: {
    text: string;
    value: string;
  }[];

  listEntityEventHeaders: {
    text: string;
    value: string;
  }[];

  // Queries
  getEntity: DocumentNode;
  listEntities: DocumentNode;
  pickComponent: DocumentNode;
  streamEntityEvents: DocumentNode;
  listEntityEvents: DocumentNode;

  // Mutations
  createEntity: DocumentNode;

  constructor(
    typeName: string,
    {
      getEntity,
      listEntities,
      pickComponent,
      streamEntityEvents,
      createEntity,
      name,
      componentProperties,
      hints,
      showProperties,
      showActions,
      listHeaders,
      listEntityEventHeaders,
      listEntityEvents,
      icon,
    }: {
      getEntity: DocumentNode;
      listEntities: DocumentNode;
      pickComponent: DocumentNode;
      streamEntityEvents: DocumentNode;
      createEntity: DocumentNode;
      name: string;
      componentProperties: string[];
      hints: SiComponent["hints"];
      showProperties: SiComponent["showProperties"];
      showActions: SiComponent["showActions"];
      listHeaders: SiComponent["listHeaders"];
      listEntityEventHeaders: SiComponent["listEntityEventHeaders"];
      listEntityEvents: SiComponent["listEntityEvents"];
      icon: string;
    },
  ) {
    this.typeName = typeName;
    this.getEntity = getEntity;
    this.listEntities = listEntities;
    this.pickComponent = pickComponent;
    this.streamEntityEvents = streamEntityEvents;
    this.createEntity = createEntity;
    this.name = name;
    this.componentProperties = componentProperties;
    this.hints = hints;
    this.showProperties = showProperties;
    this.showActions = showActions;
    this.listHeaders = listHeaders;
    this.listEntityEventHeaders = listEntityEventHeaders;
    this.icon = icon;
    this.listEntityEvents = listEntityEvents;
  }

  listEntitiesResultString(): string {
    return `${this.typeName}ListEntities`;
  }

  getEntityResultString(): string {
    return `${this.typeName}GetEntity`;
  }

  pickComponentResultString(): string {
    return `${this.typeName}PickComponent`;
  }

  streamEntityEventsResultString(): string {
    return `streamEntityEvents`;
  }

  createEntityResultString(): string {
    return `${this.typeName}CreateEntity`;
  }

  listEntityEventsResultString(): string {
    return `${this.typeName}ListEntityEvents`;
  }
}
