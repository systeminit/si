import { DocumentNode } from "graphql";

export class SiComponent {
  typeName: string;
  name: string;

  // Properties
  componentProperties: string[];

  // Hints
  hints: {
    constraintName: string;
    hintValue: string;
  }[];

  showActions: {
    displayName: string;
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

  // Queries
  getEntity: DocumentNode;
  listEntities: DocumentNode;
  pickComponent: DocumentNode;
  streamEntityEvents: DocumentNode;

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
    return `${this.typeName}StreamEntityEvents`;
  }

  createEntityResultString(): string {
    return `${this.typeName}CreateEntity`;
  }
}
