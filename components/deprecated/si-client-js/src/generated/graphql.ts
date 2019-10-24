export type Maybe<T> = T | null;
/** All built-in and custom scalars, mapped to their actual values */
export type Scalars = {
  ID: string,
  String: string,
  Boolean: boolean,
  Int: number,
  Float: number,
};

export type Component = {
  /** The ID */
  id: Scalars['ID'],
  /** The Name of the Component */
  name: Scalars['String'],
  /** Description of the element */
  description: Scalars['String'],
  /** Raw data for the component */
  rawDataJson: Scalars['String'],
  /** The integration that backs the component */
  integration: Integration,
  /** The kind of node created by this component type */
  nodeType: Scalars['String'],
  /** The list of supported actions for this component */
  supportedActions: Array<Maybe<Scalars['String']>>,
};

/** A CPU Component */
export type CpuComponent = Component & {
   __typename?: 'CpuComponent',
  /** The ID */
  id: Scalars['ID'],
  /** The name of the component */
  name: Scalars['String'],
  /** Description of the element */
  description: Scalars['String'],
  /** Raw data for the component */
  rawDataJson: Scalars['String'],
  /** The integration that backs the component */
  integration: Integration,
  /** The type of node created by this component type */
  nodeType: Scalars['String'],
  supportedActions: Array<Maybe<Scalars['String']>>,
  cores: Scalars['Int'],
  baseFreqMHz: Scalars['Int'],
  allCoreTurboFreqMHz: Scalars['Int'],
  singleCoreTurboFreqMHz: Scalars['Int'],
  architecture: Scalars['String'],
  manufacturer: Scalars['String'],
};

export type CpuEntity = {
   __typename?: 'CpuEntity',
  /** The ID */
  id: Scalars['ID'],
  /** The name of the port */
  name: Scalars['String'],
  /** A longer description of the port */
  description: Scalars['String'],
  cores: Scalars['Int'],
  baseFreqMHz: Scalars['Int'],
  allCoreTurboFreqMHz: Scalars['Int'],
  singleCoreTurboFreqMHz: Scalars['Int'],
  architecture: Scalars['String'],
  manufacturer: Scalars['String'],
  component?: Maybe<CpuComponent>,
};

export type CreateCpuArgs = {
  name?: Maybe<Scalars['String']>,
  description?: Maybe<Scalars['String']>,
};

export type CreateCpuInput = {
  constraints?: Maybe<Scalars['String']>,
  args?: Maybe<CreateCpuArgs>,
  workspace: Scalars['String'],
};

export type CreateCpuPayload = {
   __typename?: 'CreateCpuPayload',
  cpu: CpuEntity,
};

export type CreateDiskImageArgs = {
  name?: Maybe<Scalars['String']>,
  description?: Maybe<Scalars['String']>,
};

export type CreateDiskImageInput = {
  constraints?: Maybe<Scalars['String']>,
  args?: Maybe<CreateDiskImageArgs>,
  workspace: Scalars['String'],
};

export type CreateDiskImagePayload = {
   __typename?: 'CreateDiskImagePayload',
  diskImage: DiskImageEntity,
};

/** Create an Integration Instance */
export type CreateIntegrationInstanceInput = {
  /** The Integration ID you want to add */
  integrationId: Scalars['ID'],
  /** The name of this integration in your account */
  name: Scalars['String'],
  /** The description of what this integration is for */
  description: Scalars['String'],
  /** The options for using this integration in the backend */
  options: Scalars['String'],
};

/** The response to a createIntegrationInstance call */
export type CreateIntegrationInstancePayload = {
   __typename?: 'createIntegrationInstancePayload',
  /** The created integration instance */
  integrationInstance?: Maybe<IntegrationInstance>,
};

export type CreateOperatingSystemArgs = {
  /** The name of the SSH Key */
  name?: Maybe<Scalars['String']>,
  /** A description of the SSH Key */
  description?: Maybe<Scalars['String']>,
};

export type CreateOperatingSystemInput = {
  constraints?: Maybe<Scalars['String']>,
  args?: Maybe<CreateOperatingSystemArgs>,
  workspace: Scalars['String'],
};

export type CreateOperatingSystemPayload = {
   __typename?: 'CreateOperatingSystemPayload',
  operatingSystem: OperatingSystemEntity,
};

export type CreatePortArgs = {
  /** The name of the port */
  name: Scalars['String'],
  /** A longer description of the port */
  description?: Maybe<Scalars['String']>,
  /** The service name */
  serviceName?: Maybe<Scalars['String']>,
  /** The protocol */
  protocol?: Maybe<Scalars['String']>,
  /** The number of the port */
  number?: Maybe<Scalars['Int']>,
};

export type CreatePortInput = {
  constraints?: Maybe<Scalars['String']>,
  args: CreatePortArgs,
  workspace: Scalars['String'],
};

export type CreatePortPayload = {
   __typename?: 'CreatePortPayload',
  port: PortEntity,
};

export type CreateServerArgs = {
  /** The name of the Server */
  name?: Maybe<Scalars['String']>,
  /** A description of the Server */
  description?: Maybe<Scalars['String']>,
};

export type CreateServerInput = {
  constraints?: Maybe<Scalars['String']>,
  args?: Maybe<CreateServerArgs>,
  workspace: Scalars['String'],
};

export type CreateServerPayload = {
   __typename?: 'CreateServerPayload',
  server: ServerEntity,
};

export type CreateSshKeyArgs = {
  /** The name of the SSH Key */
  name?: Maybe<Scalars['String']>,
  /** A description of the SSH Key */
  description?: Maybe<Scalars['String']>,
  /** The private key */
  privateKey?: Maybe<Scalars['String']>,
  /** The public key */
  publicKey?: Maybe<Scalars['String']>,
};

export type CreateSshKeyInput = {
  constraints?: Maybe<Scalars['String']>,
  args?: Maybe<CreateSshKeyArgs>,
  workspace: Scalars['String'],
};

export type CreateSshKeyPayload = {
   __typename?: 'CreateSshKeyPayload',
  sshKey: SshKeyEntity,
};

export type CreateUserInput = {
  /** The users email */
  email: Scalars['String'],
  /** The users name */
  name?: Maybe<Scalars['String']>,
};

/** The response to a createUser mutation */
export type CreateUserPayload = {
   __typename?: 'createUserPayload',
  /** The created user */
  user?: Maybe<User>,
};

export type CreateWorkspaceInput = {
  /** The name of the workspace */
  name: Scalars['String'],
  /** What the workspace is for */
  description?: Maybe<Scalars['String']>,
};

/** The result of a createWorkspace mutation */
export type CreateWorkspacePayload = {
   __typename?: 'createWorkspacePayload',
  /** The workspace that was just created */
  workspace?: Maybe<Workspace>,
};

/** Delete an Integration Instance */
export type DeleteIntegrationInstanceInput = {
  /** The ID fo the Integration Instance to delete */
  id: Scalars['ID'],
};

/** The result of a deleteIntegrationInstance mutation */
export type DeleteIntegrationInstancePayload = {
   __typename?: 'deleteIntegrationInstancePayload',
  /** The deleted integration instance */
  integrationInstance?: Maybe<IntegrationInstance>,
};

export type DeleteWorkspaceInput = {
  /** The ID of the Workspace to delete */
  id: Scalars['ID'],
};

/** The result of a deleteWorkspace mutation */
export type DeleteWorkspacePayload = {
   __typename?: 'deleteWorkspacePayload',
  /** The workspace ID that was just created */
  workspace?: Maybe<Workspace>,
};

/** Disable an Integration Instance on a Workspace */
export type DisableIntegrationInstanceOnWorkspaceInput = {
  /** The ID for the integration instance to enable */
  integrationInstanceId: Scalars['ID'],
  /** The ID for the workspace to enable */
  workspaceId: Scalars['ID'],
};

/** The response to enabling an integration instance on a workspace */
export type DisableIntegrationInstanceOnWorkspacePayload = {
   __typename?: 'disableIntegrationInstanceOnWorkspacePayload',
  /** The associated integration instance */
  integrationInstance?: Maybe<IntegrationInstance>,
  /** The workspace */
  workspace?: Maybe<Workspace>,
};

/** A Disk Image Component */
export type DiskImageComponent = Component & {
   __typename?: 'DiskImageComponent',
  /** The ID */
  id: Scalars['ID'],
  /** The name of the component */
  name: Scalars['String'],
  /** Description of the element */
  description: Scalars['String'],
  /** Raw data for the component */
  rawDataJson: Scalars['String'],
  /** The integration that backs the component */
  integration: Integration,
  /** The type of node created by this component type */
  nodeType: Scalars['String'],
  /** The format of this disk image */
  format: Scalars['String'],
  /** The operating system inside this image */
  operatingSystem?: Maybe<OperatingSystemComponent>,
  supportedActions: Array<Maybe<Scalars['String']>>,
};

/** A Disk Image Component */
export type DiskImageEntity = {
   __typename?: 'DiskImageEntity',
  /** The ID */
  id: Scalars['ID'],
  /** The name of the disk image */
  name: Scalars['String'],
  /** Description of the disk image */
  description: Scalars['String'],
  /** The format of this disk image */
  format: Scalars['String'],
  /** The operating system inside this image */
  operatingSystem?: Maybe<OperatingSystemEntity>,
  /** The optional component */
  component?: Maybe<DiskImageComponent>,
};

/** Enable an Integration Instance on a Workspace */
export type EnableIntegrationInstanceOnWorkspaceInput = {
  /** The ID for the integration instance to enable */
  integrationInstanceId: Scalars['ID'],
  /** The ID for the workspace to enable */
  workspaceId: Scalars['ID'],
};

/** The response to enabling an integration instance on a workspace */
export type EnableIntegrationInstanceOnWorkspacePayload = {
   __typename?: 'enableIntegrationInstanceOnWorkspacePayload',
  /** The associated integration instance */
  integrationInstance?: Maybe<IntegrationInstance>,
  /** The workspace */
  workspace?: Maybe<Workspace>,
};

/** Find a component with a searchjs query */
export type FindComponentInput = {
  /** A workspace to limit the search to */
  workspace?: Maybe<Scalars['String']>,
  /** A JSON SearchJS Query */
  search: Scalars['String'],
};

export type GetComponentsInput = {
  integration?: Maybe<Scalars['String']>,
  workspace?: Maybe<Scalars['String']>,
};

export type GetIntegrationInstanceByIdInput = {
  /** The ID for the Integration Instance */
  id: Scalars['ID'],
};

export type GetUserByIdInput = {
  /** The ID for the user */
  id: Scalars['ID'],
};

export type GetWorkspaceByIdInput = {
  /** The ID for the Workspace */
  id: Scalars['ID'],
};

/** An Integration */
export type Integration = {
   __typename?: 'Integration',
  /** The ID */
  id: Scalars['ID'],
  /** The name of the service */
  name: Scalars['String'],
  /** Description of the service */
  description?: Maybe<Scalars['String']>,
  /** The options for the integration */
  options?: Maybe<IntegrationOption>,
  /** The image for the integration */
  image?: Maybe<Scalars['String']>,
};

/** An instance of an integration, created by a user */
export type IntegrationInstance = {
   __typename?: 'IntegrationInstance',
  /** The ID of this Integration Instance */
  id: Scalars['ID'],
  /** The integration */
  integration: Integration,
  /** The user who created this integration */
  user: User,
  /** The name of this integration */
  name: Scalars['String'],
  /** The description of this integration */
  description: Scalars['String'],
  /** The options for this integration */
  options: Scalars['String'],
  /** Workspaces this integration instance is enabled on */
  workspaces: Array<Maybe<Workspace>>,
};

export type IntegrationOption = {
   __typename?: 'IntegrationOption',
  fields?: Maybe<Array<Maybe<IntegrationOptionField>>>,
};

export type IntegrationOptionField = {
   __typename?: 'IntegrationOptionField',
  id?: Maybe<Scalars['String']>,
  name?: Maybe<Scalars['String']>,
  type?: Maybe<Scalars['String']>,
};

export type Mutation = {
   __typename?: 'Mutation',
  /** Create a new user */
  createUser?: Maybe<CreateUserPayload>,
  /** Create a new Workspace */
  createWorkspace?: Maybe<CreateWorkspacePayload>,
  /** Delete a Workspace */
  deleteWorkspace?: Maybe<DeleteWorkspacePayload>,
  /** Create a new integration */
  createIntegrationInstance?: Maybe<CreateIntegrationInstancePayload>,
  /** Delete an integration instance */
  deleteIntegrationInstance?: Maybe<DeleteIntegrationInstancePayload>,
  /** Enable an integration instance on a workspace */
  enableIntegrationInstanceOnWorkspace?: Maybe<EnableIntegrationInstanceOnWorkspacePayload>,
  /** Disable an integration instance on a workspace */
  disableIntegrationInstanceOnWorkspace?: Maybe<DisableIntegrationInstanceOnWorkspacePayload>,
  createServer: CreateServerPayload,
  createOperatingSystem: CreateOperatingSystemPayload,
  createDiskImage: CreateDiskImagePayload,
  createCpu: CreateCpuPayload,
  createPort: CreatePortPayload,
  createSshKey: CreateSshKeyPayload,
};


export type MutationCreateUserArgs = {
  input?: Maybe<CreateUserInput>
};


export type MutationCreateWorkspaceArgs = {
  input?: Maybe<CreateWorkspaceInput>
};


export type MutationDeleteWorkspaceArgs = {
  input?: Maybe<DeleteWorkspaceInput>
};


export type MutationCreateIntegrationInstanceArgs = {
  input?: Maybe<CreateIntegrationInstanceInput>
};


export type MutationDeleteIntegrationInstanceArgs = {
  input?: Maybe<DeleteIntegrationInstanceInput>
};


export type MutationEnableIntegrationInstanceOnWorkspaceArgs = {
  input?: Maybe<EnableIntegrationInstanceOnWorkspaceInput>
};


export type MutationDisableIntegrationInstanceOnWorkspaceArgs = {
  input?: Maybe<DisableIntegrationInstanceOnWorkspaceInput>
};


export type MutationCreateServerArgs = {
  input?: Maybe<CreateServerInput>
};


export type MutationCreateOperatingSystemArgs = {
  input?: Maybe<CreateOperatingSystemInput>
};


export type MutationCreateDiskImageArgs = {
  input?: Maybe<CreateDiskImageInput>
};


export type MutationCreateCpuArgs = {
  input?: Maybe<CreateCpuInput>
};


export type MutationCreatePortArgs = {
  input?: Maybe<CreatePortInput>
};


export type MutationCreateSshKeyArgs = {
  input?: Maybe<CreateSshKeyInput>
};

/** An Operating System component */
export type OperatingSystemComponent = Component & {
   __typename?: 'OperatingSystemComponent',
  /** The ID */
  id: Scalars['ID'],
  /** The name of the component */
  name: Scalars['String'],
  /** Description of the element */
  description: Scalars['String'],
  /** Raw data for the component */
  rawDataJson: Scalars['String'],
  /** The integration that backs the component */
  integration: Integration,
  /** The type of node created by this component type */
  nodeType: Scalars['String'],
  /** The name of the operating system */
  operatingSystemName: Scalars['String'],
  /** The version of the operating system */
  operatingSystemVersion: Scalars['String'],
  /** The release of the operating system */
  operatingSystemRelease: Scalars['String'],
  /** The name of the operating systems platform */
  platform: Scalars['String'],
  /** The platform version */
  platformVersion: Scalars['String'],
  /** The release of the platform */
  platformRelease: Scalars['String'],
  /** The system architectures */
  architecture?: Maybe<Array<Scalars['String']>>,
  /** The list of supported actions that can be taken */
  supportedActions: Array<Maybe<Scalars['String']>>,
  diskImages: Array<Maybe<DiskImageComponent>>,
};

/** An Operating System entity */
export type OperatingSystemEntity = {
   __typename?: 'OperatingSystemEntity',
  /** The ID */
  id: Scalars['ID'],
  /** The name of the component */
  name: Scalars['String'],
  /** Description of the element */
  description: Scalars['String'],
  /** The name of the operating system */
  operatingSystemName: Scalars['String'],
  /** The version of the operating system */
  operatingSystemVersion: Scalars['String'],
  /** The release of the operating system */
  operatingSystemRelease: Scalars['String'],
  /** The name of the operating systems platform */
  platform: Scalars['String'],
  /** The platform version */
  platformVersion: Scalars['String'],
  /** The release of the platform */
  platformRelease: Scalars['String'],
  /** The system architectures */
  architecture: Scalars['String'],
  /** An optional component this entity was created with */
  component?: Maybe<OperatingSystemComponent>,
};

/** A Port Component */
export type PortComponent = Component & {
   __typename?: 'PortComponent',
  /** The ID */
  id: Scalars['ID'],
  /** The name of the component */
  name: Scalars['String'],
  /** Description of the element */
  description: Scalars['String'],
  /** Raw data for the component */
  rawDataJson: Scalars['String'],
  /** The integration that backs the component */
  integration: Integration,
  /** The type of node created by this component type */
  nodeType: Scalars['String'],
  /** The supported actions on this component */
  supportedActions: Array<Maybe<Scalars['String']>>,
  /** The service name */
  serviceName: Scalars['String'],
  /** The protocol of the port */
  protocol: Scalars['String'],
  /** The number of the port */
  number: Scalars['Int'],
};

export type PortEntity = {
   __typename?: 'PortEntity',
  /** The ID */
  id: Scalars['ID'],
  /** The name of the port */
  name: Scalars['String'],
  /** A longer description of the port */
  description: Scalars['String'],
  /** The service name */
  serviceName: Scalars['String'],
  /** The protocol */
  protocol: Scalars['String'],
  /** The number of the port */
  number: Scalars['Int'],
  /** An optional component this entity was created with */
  component?: Maybe<PortComponent>,
};

export type Query = {
   __typename?: 'Query',
  /** test message */
  testMessage: Scalars['String'],
  /** Get a User by their ID */
  getUserById?: Maybe<User>,
  /** Get Workspace by ID */
  getWorkspaceById?: Maybe<Workspace>,
  /** Gets the logged in users Workspaces */
  getWorkspaces: Array<Maybe<Workspace>>,
  /** Get a list of all known integrations */
  getAllIntegrations: Array<Maybe<Integration>>,
  /** Get a list of all this users integration instances */
  getIntegrationInstances: Array<Maybe<IntegrationInstance>>,
  /** Get a specific integration instance */
  getIntegrationInstanceById?: Maybe<IntegrationInstance>,
  /** Get a list of all known components */
  getComponents: Array<Maybe<Component>>,
  /** Find components */
  findComponents: Array<Maybe<Component>>,
  /** Get Server Components enabled for this user */
  getServerComponents: Array<Maybe<ServerComponent>>,
  findServerComponents: Array<Maybe<ServerComponent>>,
  /** Get Operating System Components enabled for this user */
  getOperatingSystemComponents: Array<Maybe<OperatingSystemComponent>>,
  findOperatingSystemComponents: Array<Maybe<OperatingSystemComponent>>,
  /** Get Operating System Components enabled for this user */
  getDiskImageComponents: Array<Maybe<DiskImageComponent>>,
  findDiskImageComponents: Array<Maybe<DiskImageComponent>>,
  /** Get Operating System Components enabled for this user */
  getCpuComponents: Array<Maybe<CpuComponent>>,
  findCpuComponents: Array<Maybe<CpuComponent>>,
  /** Get Operating System Components enabled for this user */
  getPortComponents: Array<Maybe<PortComponent>>,
  findPortComponents: Array<Maybe<PortComponent>>,
  getSshKeyComponents: Array<Maybe<SshKeyComponent>>,
  findSshKeyComponents: Array<Maybe<SshKeyComponent>>,
};


export type QueryGetUserByIdArgs = {
  input?: Maybe<GetUserByIdInput>
};


export type QueryGetWorkspaceByIdArgs = {
  input?: Maybe<GetWorkspaceByIdInput>
};


export type QueryGetIntegrationInstanceByIdArgs = {
  input?: Maybe<GetIntegrationInstanceByIdInput>
};


export type QueryGetComponentsArgs = {
  where?: Maybe<GetComponentsInput>
};


export type QueryFindComponentsArgs = {
  where: FindComponentInput
};


export type QueryGetServerComponentsArgs = {
  where?: Maybe<GetComponentsInput>
};


export type QueryFindServerComponentsArgs = {
  where?: Maybe<FindComponentInput>
};


export type QueryGetOperatingSystemComponentsArgs = {
  where?: Maybe<GetComponentsInput>
};


export type QueryFindOperatingSystemComponentsArgs = {
  where?: Maybe<FindComponentInput>
};


export type QueryGetDiskImageComponentsArgs = {
  where?: Maybe<GetComponentsInput>
};


export type QueryFindDiskImageComponentsArgs = {
  where?: Maybe<FindComponentInput>
};


export type QueryGetCpuComponentsArgs = {
  where?: Maybe<GetComponentsInput>
};


export type QueryFindCpuComponentsArgs = {
  where?: Maybe<FindComponentInput>
};


export type QueryGetPortComponentsArgs = {
  where?: Maybe<GetComponentsInput>
};


export type QueryFindPortComponentsArgs = {
  where?: Maybe<FindComponentInput>
};


export type QueryGetSshKeyComponentsArgs = {
  where?: Maybe<GetComponentsInput>
};


export type QueryFindSshKeyComponentsArgs = {
  where?: Maybe<FindComponentInput>
};

export type ServerComponent = Component & {
   __typename?: 'ServerComponent',
  /** The ID */
  id: Scalars['ID'],
  /** The Name of the Component */
  name: Scalars['String'],
  /** Description of the element */
  description: Scalars['String'],
  /** Raw data for the component */
  rawDataJson: Scalars['String'],
  /** The integration that backs the component */
  integration: Integration,
  /** The type of node created by this component type */
  nodeType: Scalars['String'],
  /** The amount of memory */
  memoryGIB: Scalars['Int'],
  /** The number of CPU cores */
  cpuCores?: Maybe<Scalars['Int']>,
  /** The list of supported actions that can be taken */
  supportedActions: Array<Maybe<Scalars['String']>>,
  cpu: CpuComponent,
};

/** An Operating System entity */
export type ServerEntity = {
   __typename?: 'ServerEntity',
  /** The ID */
  id: Scalars['ID'],
  /** The name of the component */
  name: Scalars['String'],
  /** Description of the element */
  description: Scalars['String'],
  /** The amount of memory */
  memoryGIB: Scalars['Int'],
  /** The number of CPU cores */
  cpuCores?: Maybe<Scalars['Int']>,
  /** CPU */
  cpu?: Maybe<CpuEntity>,
  /** SSH Key */
  sshKey?: Maybe<SshKeyEntity>,
  /** Operating System */
  operatingSystem?: Maybe<OperatingSystemEntity>,
  /** The component this entity was created from */
  component?: Maybe<ServerComponent>,
};

/** An SSH Key Component */
export type SshKeyComponent = Component & {
   __typename?: 'SshKeyComponent',
  /** The ID */
  id: Scalars['ID'],
  /** The name of the component */
  name: Scalars['String'],
  /** Description of the element */
  description: Scalars['String'],
  /** Raw data for the component */
  rawDataJson: Scalars['String'],
  /** The integration that backs the component */
  integration: Integration,
  /** The type of node created by this component type */
  nodeType: Scalars['String'],
  /** The supported actions on this component */
  supportedActions: Array<Maybe<Scalars['String']>>,
  /** The type of keys */
  keyType: Scalars['String'],
  /** The format of the keys */
  keyFormat: Scalars['String'],
  /** The number of bits in the key */
  bits: Scalars['Int'],
};

/** An SSH Key Entity */
export type SshKeyEntity = {
   __typename?: 'SshKeyEntity',
  /** The ID */
  id: Scalars['ID'],
  /** The name of the SSH Key */
  name: Scalars['String'],
  /** A longer description of the SSH Key */
  description: Scalars['String'],
  /** The type of keys */
  keyType: Scalars['String'],
  /** The format of the keys */
  keyFormat: Scalars['String'],
  /** The number of bits in the keys */
  bits: Scalars['String'],
  /** The comment in the keys */
  comment: Scalars['String'],
  /** The bubble babble fingerprint of the keys */
  bubbleBabble: Scalars['String'],
  /** The sha256 fingerprint of the keys */
  fingerPrint: Scalars['String'],
  /** The randomArt fingerprint of the keys */
  randomArt: Scalars['String'],
  /** The private key */
  privateKey: Scalars['String'],
  /** The public key */
  publicKey: Scalars['String'],
  /** An optional component this entity was created with */
  component?: Maybe<SshKeyComponent>,
};

/** A User */
export type User = {
   __typename?: 'User',
  /** The id */
  id: Scalars['ID'],
  /** The Email Address reported by their provider */
  email: Scalars['String'],
  /** The Name reported by their provider */
  name?: Maybe<Scalars['String']>,
  /** The workspaces this user is a member of */
  workspaces?: Maybe<Array<Maybe<Workspace>>>,
  /** The workspaces this user is the creator of */
  createdWorkspaces?: Maybe<Array<Maybe<Workspace>>>,
  /** Integration Instances created by this user */
  integrationInstances: Array<Maybe<IntegrationInstance>>,
};

/** A workspace; where work happens */
export type Workspace = {
   __typename?: 'Workspace',
  /** The id */
  id: Scalars['ID'],
  /** The name of the workspace */
  name: Scalars['String'],
  /** Description of what the workspace is for */
  description: Scalars['String'],
  /** Members of the workspace */
  members: Array<User>,
  /** Creator of the workspace */
  creator: User,
  /** The integration instances enabled for this workspace */
  integrationInstances: Array<Maybe<IntegrationInstance>>,
};
