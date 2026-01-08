# Changelog

All notable changes to System Initiative will be documented in this file.

## December 22nd 2025 to January 4th, 2026

### ✨ Added

- Added support for **Cursor** in the SI AI Agent.
- Added the ability to **upgrade a component** from the CLI.
- Added an option to **restart workspace onboarding** in the Web App.
- Added the ability to **invite users to a workspace** via the CLI.

### 🔄 Changed

- Reduced AI Agent token usage through improved caching and prompt updates.

<iframe width="560" height="315" src="https://www.youtube.com/embed/afE-1yH9cG8?si=1_c_NJ3HUBp5-K-5" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" referrerpolicy="strict-origin-when-cross-origin" allowfullscreen></iframe>

## December 15th to 19th, 2025

### ✨ Added

- Added the ability to **leave a workspace** through the Public API and Auth
  Portal.
- Added support for `--open` on change set commands in the SI Binary to open a
  change set.
- Added support for passing a **workspace name** to `si workspace switch` in the
  SI Binary.
- Added the ability to **capture request and response logs** from commands in
  the SI Binary.
- Added support in the Public API for identifying **which secret a component is
  connected to**.
- Added support for passing a **name** to `si secret create` and
  `si secret update` in the SI Binary.
- Added the ability to **contribute schemas via the CLI**.

### 🔄 Changed

- Updated how **secrets are updated** via the SI Binary.

### 🐞 Fixed

- Fixed an issue where creating a secret in the CLI errored if the secret wasn't
  installed.
- Fixed incorrect rendering of **nested action dependencies** in the
  ActionsPanel of the Web App.
- Fixed an issue where the **actions panel** sometimes showed the wrong selected
  action.
- Fixed a broken **bulk edit** keyboard shortcut in the Web App.
- Fixed an issue where authoring a **schema change** could break the schema.
- Fixed an issue where multiselecting deleted and non-deleted components
  prevented using the **erase** context menu in the Web App.
- Fixed unnecessary restrictions on contributing **schema changes** in Luminork.
- Fixed an issue in the Web App to avoid **graph cycles** in the action list.
- Fixed an issue causing **duplicate actions** from other change sets on the
  same component.
- Fixed change set **apply exit logic** in the SI Binary.
- Fixed an issue where writing **AI agent files and templates** was broken.

## December 8th to 12th, 2025

### ✨ Added

- Launched **Microsoft Entra assets**.
- Added support for **OpenCode.ai** to the AI agent.
- Added the ability to **authenticate to System Initiative** via the CLI.
- Added the ability to **navigate to a change set** from the CLI.
- Added the ability to **update a secret** from the CLI.
- Added the ability to **apply a change set** and view running actions from the
  CLI.
- Added the ability to **reference shared functions** when authoring via the
  CLI.

### 🔄 Changed

- Deprecated the **Docker-based SI MCP server**.
- Improved **workspace selection** behavior in the CLI.

### 🐞 Fixed

- Fixed an issue where the **secret create** command did not respect the API
  token environment variable in the SI CLI.
- Fixed an issue where the **ApplyChangeSetModal** title scrolled incorrectly in
  the Web App.
- Fixed an issue where running templates did not trigger the **authentication
  flow** in the CLI.

<iframe width="560" height="315" src="https://www.youtube.com/embed/5gEe4l6xzyc?si=cHNTIkp9jbOM0AS3" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" referrerpolicy="strict-origin-when-cross-origin" allowfullscreen></iframe>

## December 2nd to 7th, 2025

### ✨ Added

- Added support for generating templates in the SI binary.
- Added the ability to create secrets from the SI CLI.
- Added support for fetching remote templates via the CLI.
- Added change set management capabilities to the SI CLI.

### 🔄 Changed

- Ensured automation tokens are revoked when a workspace is deleted.
- Updated the styling of the Actions panel in the Web App.

### 🐞 Fixed

- Fixed an issue where `NotFound` components were not treated as 404 responses
  in the Public API.
- Fixed an issue hiding hidden parent props in the Web App Attribute Panel.
- Fixed indexing issues in the Web App with SQLite to support multiple tabs
  correctly.
- Fixed an issue where the AI Agent did not correctly write Codex configuration
  to disk.
- Fixed outdated function run logs in the Web App.

<iframe width="560" height="315" src="https://www.youtube.com/embed/saN-K5Kay8g?si=xNJ81ga20rQXEx42" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" referrerpolicy="strict-origin-when-cross-origin" allowfullscreen></iframe>

## November 24th to 30th, 2025

### ✨ Added

- Added the option to skip creating overlays when using the SI binary for
  authoring.
- Added workspace filtering on the Web App home page.
- Added the ability to contribute modules through the Public API.
- Released a new SI binary in the artifacts store.
- Added support for OpenAI Codex in the AI Agent (available via the SI binary).
- Introduced Unix and Windows installers for the SI binary.

### 🔄 Changed

- Improved component import workflows in the Web Application.
- Removed template creation from both the Web App and the Public API.

### 🐞 Fixed

- Fixed an Auth API issue where workspace names containing domains were
  incorrectly sent as links.
- Fixed an onboarding flow issue in the Web App when using the override option.

<iframe width="560" height="315" src="https://www.youtube.com/embed/2wmX-Ss-cfk?si=BA-HDZxQcrMHdexN" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" referrerpolicy="strict-origin-when-cross-origin" allowfullscreen></iframe>

## November 17th to 23rd, 2025

### ✨ Added

- Launched coverage for **DigitalOcean assets** with new onboarding flow.
- Added support for **pulling uninstalled schemas** in the SI CLI.
- Added an **installed schema event** to the workspace audit log.
- Added the ability to **skip creating overlays** when editing built-ins in the
  Public API.

### 🔄 Changed

- Improved consistency between the **Attribute and Secret Panels** in the web
  app.
- Allowed users to **switch workspaces** while waiting in the lobby.
- Removed the automatic redirect to a **default workspace** during login.

### 🐞 Fixed

- Fixed an issue with **bulk writes to SQLite** in the web app.
- Fixed map rendering issues in the web app.
- Fixed poor rendering of the **code viewer** when displaying a single line of
  code.
- Fixed onboarding screen rendering issues in **Firefox**.
- Fixed an issue in the MCP Server where `componentDiscover` selected the wrong
  function type.
- Fixed schema color handling when authoring via the **SI binary**.
- Fixed an issue creating a **template from baseline** in the SI binary.

<iframe width="560" height="315" src="https://www.youtube.com/embed/J-bAGaiJoPk?si=n9_KXq29W6rVxI-4" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" referrerpolicy="strict-origin-when-cross-origin" allowfullscreen></iframe>

## November 10th to 16th, 2025

### ✨ Added

- Added filtering in the Web App Map view to show components with diffs.
- Added support for retrieving component resources via the Public API.
- Added the ability to execute debug functions through the Public API.
- Added support for deleting overlay function bindings and updating overlay
  functions via the Public API.
- Added support for conduit to push and pull overlay functions.
- Added support for conduit to push schema names instead of all schemas.
- Added the ability to create attribute and transformation functions via the
  Public API.
- Added visibility in the Web App when an AI Agent creates a change set.
- Added support for pulling overlays and schemas using wildcards in conduit.
- Added support for using debug functions through the MCP tool.

### 🔄 Changed

- Improved SQLite performance by adding a ReadWrite lock.
- Removed the requirement for the AI Agent banner after onboarding in the Web
  App.

### 🐞 Fixed

- Fixed unnecessary SQLite queries triggered during Web App page load.
- Fixed an issue where component upgrades incorrectly queued update functions.
- Fixed an issue where the component details page briefly showed an incorrect
  “does not exist” message.
- Fixed collapsing behavior for categories in the AddComponentModal.
- Fixed an error not being captured when creating a component in the
  AddComponentModal.
- Fixed multiple issues in the right-click context menu related to hotkeys and
  menu visibility.

<iframe width="560" height="315" src="https://www.youtube.com/embed/nFJgHwkR4kY?si=Zt-yT4SZiDljrRIu" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" referrerpolicy="strict-origin-when-cross-origin" allowfullscreen></iframe>

## November 3rd to 9th, 2025

### ✨ Added

- Added support for creating **overlay qualification and codegen functions** in
  the Public API.
- Exposed the **workspace ExternalId** in Public API workspace operations.
- Supported **Codegen and Qualification overlay functions** in `si-mcp-server`.
- Created an **initial API token** when creating a workspace via the Public API.
- Allowed creation of **Auth API workspace tokens** via the automation API.
- Added the ability to **write property and unit tests for function types**.
- Public API now returns whether a **function is an overlay**.
- Public API now **creates an initial workspace API token** when creating a
  workspace.
- Launched **Azure beta assets and onboarding** in the web app.
- Added the ability to **create and manipulate templating outside of the web
  app**.

### 🔄 Changed

- Improved handling of **workspace snapshot evictions** to avoid race
  conditions.
- Changed the **Func Run log viewer** to show plain text for very large logs.
- Applied **UI consistency updates** for colors and borders.
- Changed the **Add Component modal** in the web app to be virtualized for
  better performance.

### 🐞 Fixed

- Fixed an issue where **queued actions on HEAD** also appeared on a change set.
- Fixed an issue where **Conduit unlocked all schemas** even when no changes
  were pending.
- Fixed import functions that were **not working correctly for Azure
  components**.
- Fixed path handling in the **Attributes API** for map items and keys that
  contained `/`.
- Fixed an issue where **deletions or upgrades** could break components with
  subscriptions.
- Fixed an issue where **default subscriptions were not preserved** during a
  component upgrade.
- Fixed an issue where **variant functions in the Public API** did not indicate
  overlay status.
- Fixed an issue where **deployment MVs were not cleaned up** when removed from
  the index.
- Fixed an issue where **prop validation failures** showed for subscriptions
  that were not yet resolved in the web app.
- Fixed **scroll jitter** on the component review page in the web app.
- Fixed an issue where **resetting Attribute Panel inputs** prevented default
  behavior from running.
- Fixed an issue where **selecting components on the map view** caused a very
  jittery experience.

<iframe width="560" height="315" src="https://www.youtube.com/embed/1PjmaXFXR50?si=gKSwqKYsS51gRIqO" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" referrerpolicy="strict-origin-when-cross-origin" allowfullscreen></iframe>

## October 27th to November 2nd, 2025

### ✨ Added

- Added ability to **install schemas** via the Public API.
- Added a **feature flag** enabling **Azure assets** for the public beta.
- Added ability to **filter possible subscriptions** by component and schema in
  the web app.
- Added ability to **import and discover Azure resources** in the MCP server.
- Added support for **authentication functions** in Conduit.
- Added display of **previous attribute values** in the Attributes Panel.

### 🔄 Changed

- Improved **workspace snapshot eviction** logic to avoid race conditions.
- Updated **func run log viewer** to automatically display plain text for very
  large logs.
- Refined **UI consistency** across colors and borders for a cleaner experience.
- Updated `set_ai_agent_executed` endpoint in the Public API to return a **204
  status code**.
- Updated **MCP attribute tools** to no longer be hardcoded for AWS.
- Allowed **re-using function names** when attaching them to different schemas.
- Improved **Public API search** to allow case-insensitive schema name lookups.
- Updated **layer-cache** to skip NATS publishing when retrying writes.
- Allowed **editing create-only properties** on imported components within a
  change set.

### 🐞 Fixed

- Fixed an issue where the **Func Run Details page** made too many requests for
  data.
- Fixed an issue where **overlay functions** weren’t available to users in the
  web app.
- Fixed an issue where the **debug panel** in the component details page didn’t
  refresh after component changes.
- Fixed orphaned **update actions** in the web app.
- Fixed navigation in the web app where **jumping to a component from a view**
  redirected back to the default view.
- Fixed an issue where the **refresh button** wasn’t consistently available in
  all scenarios.
- Fixed an issue where the **Apply Change Set button** wasn’t properly disabled
  while loading.
- Fixed **database connection eviction logic** to close idle and long-lived
  connections correctly.

<iframe width="560" height="315" src="https://www.youtube.com/embed/lA8BS7rMJYM?si=WqLznCUgmfWsLkLQ" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" referrerpolicy="strict-origin-when-cross-origin" allowfullscreen></iframe>

## October 20th to 26th 2025

### ✨ Added

- 🚀 **Initial release of `si-conduit`**, a new tool for authoring schemas via
  files on disk.
- Added ability to **delete function attachments** via the Public API.
- Added ability to **search in the views dropdown list** in the web app.
- Added ability to **view full history and function executions** for a component
  via the Component Details page.
- Added support to **manage System Initiative workspaces** via the Public API.
- Added ability to **subscribe to another prop on the same component**.

### 🔄 Changed

- Enhanced **cache-busting logic** for Deployment MVs in the web app.
- Enhanced **Auth API workspace routes** to be fully managed by API tokens.
- Increased the **module upload size limit** in SDF.
- Removed the need for **cycle checks** in management edge creation.
- Retired **`si-fs`** as an active product.

### 🐞 Fixed

- Fixed the **MCP `validateCredentials` tool** to remove token-specific
  information.
- Fixed issues with **explore controls and context menu** in the web app.
- Fixed a bug in the **create template modal** in the web app.
- Fixed an issue where **index retrieval errors** in the web app resulted in 500
  errors to users.
- Fixed an issue where the web app now **alerts users when a change set can’t
  merge** due to DVU still in progress.
- Fixed a bug where **old schema versions** appeared in the Add Component modal.
- Fixed an issue where **management function runs** tried to load for invalid
  components.
- Fixed navigation issues when **bulk editing on HEAD**.
- Fixed malformed user input handling in the **Attributes Panel**.
- Fixed an issue in the **Auth Portal** where users could navigate to a
  workspace without a valid token.
- Fixed an issue where **failing to fetch change sets** caused users to get
  stuck in the lobby.
- Fixed error handling in the **Attributes Panel** for greater stability.
- Fixed issues preventing **incomplete subscriptions** in the product.
- Fixed an issue where **failed functions** didn’t send an update event to
  indicate failure.

<iframe width="560" height="315" src="https://www.youtube.com/embed/bQIthgGT3KY?si=lOg935rZje-eGw15" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" referrerpolicy="strict-origin-when-cross-origin" allowfullscreen></iframe>

## October 13th to 19th 2025

### ✨ Added

- Added ability to **control management functions via overlays**.
- Added support for **moving a list of components to a view** via the Public
  API.
- Added support for **`useWorkingCopy`** in `create_component` requests via the
  Public API.

### 🔄 Changed

- Updated **Hetzner Cloud server and SSH key assets** for better
  interoperability.
- Updated **MCP server** to use the new Search API for template generation and
  component upgrade tools.
- When adding **actions or management functions** to schemas (via the Public
  API) not owned by you, they are now **attached as overlay functions**.
- Persisted **Deployment MVs in SQLite** for better stability.
- Changed the **Web App** to use DeploymentMVs for schema updates when module
  indexes change.
- Enhanced **error messages** to make diagnosing PG connection closures easier.

### 🐞 Fixed

- Fixed an issue where the **rebaser retried fetching missing snapshots**.
- Fixed a confusing UX quirk where the **Workspace Token** was used for two
  different purposes.
- Fixed an issue where the **Public API** didn’t trigger an MV build when
  unlocking a schema.
- Fixed an issue where **pinning a component** didn’t work correctly.
- Fixed an issue where the **MCP tool mistakenly generated a template** when
  listing components.
- Fixed a vulnerability where **Workspace Integrations** could be accessed by
  another user without permission.

<iframe width="560" height="315" src="https://www.youtube.com/embed/WvfQnJ1NhWk?si=f4lDA2Y4P5NANRgj" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" referrerpolicy="strict-origin-when-cross-origin" allowfullscreen></iframe>

## October 6th to 12th 2025

### ✨ Added

- Added a new **Search API endpoint** to the Public API.
- Updated the **Qualifications function API endpoint** in the Public API to
  allow using secrets.
- Added ability to **collapse panels** in the Func Run Details page.
- Added ability to **search by `isUpgradable` and `hasDiff`** via the Public
  API.
- Enabled **MCP tools for authoring schemas and functions**.
- Added ability to **select all components in a group** and **multi-select
  ranges of components** in the web app.
- Added **Hetzner** to the Add Component filter list.
- Added ability to **unlock a function** via the Public API.
- Added ability to **edit a template and its associated management function**
  via the MCP server.

### 🔄 Changed

- Updated the **snapshot eviction logic** to remove the grace period.
- Optimized the frontend to use **bulk queries for patches**.
- Displayed **`schemaId` and `schemaVariantId`** in the component debug panel.
- Updated **onboarding flow** to skip for invited users if someone in the
  workspace has already completed it.
- Changed **web app patching priority** based on user activity.
- Ran **compute validation jobs in parallel** for faster processing.
- Updated **hover and selected states** in GridTiles for better usability.
- Removed redundant **buttons that reopened the onboarding flow**.

### 🐞 Fixed

- Fixed an issue where **possible connections** weren’t cache-busted correctly.
- Fixed issues where **Map and Explore** in the web UI conflicted over query
  string parameters.
- Fixed an issue when **importing or discovering Hetzner Cloud servers**.
- Fixed issues with **left and right arrow key navigation** in the Add Component
  modal.
- Fixed an issue where **onboarding didn’t trigger for new users**.
- Fixed **increased retry intervals** in Actions to prevent overloading.
- Fixed an issue where **autosubscribe could create graph cycles**.
- Fixed unnecessary **workspace snapshot loading** to improve performance.

<iframe width="560" height="315" src="https://www.youtube.com/embed/gGQEAA-dBjg?si=QUd8IWgfE1ACa7-9" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" referrerpolicy="strict-origin-when-cross-origin" allowfullscreen></iframe>

## September 29th to October 5th 2025

### ✨ Added

- Added ability to **create action functions as overlays to schemas**.
- Added a **Hetzner Cloud asset pipeline**.
- Added **WsEvents** to the Public API for creating functions to keep the web
  app updated.
- Updated the Public API to **install a schema variant automatically** if not
  installed when unlocking.
- Added extra **telemetry information** to PgPool connection eviction logic.
- Added support for **Hetzner Cloud**.
- Returned **workspace details** when a user calls the API `whoami` endpoint.
- Exposed the **function kinds** available for each type of function in a schema
  variant.

### 🔄 Changed

- Updated the **MCP server** to adhere to correct API types for optional
  parameters.
- Updated **DeploymentMVs** to only build the default variant for a schema.
- Changed **AWS assets** so refresh functions no longer trigger API limits on
  actions, and remove the resource if the refresh fails with `ResourceNotFound`.
- Allowed users to use **special characters in workspace names** in the Auth
  Portal.
- Allowed the **rebaser** to trigger request retry loops for specific error
  messages.

### 🐞 Fixed

- Fixed an issue where the **web app** showed the AI Agent banner if the wrong
  token was retrieved.
- Fixed an issue in the **Auth Portal** where the save button text was incorrect
  for manually verified users.
- Fixed misleading behavior in the **Auth Portal** where setting `1m` was
  interpreted as 1 minute, not 1 month.
- Fixed an issue where the **web app retried 500/503 index responses** properly.
- Fixed an issue where restoring a component from the **component details page
  on HEAD** didn’t redirect to the new change set.
- Fixed an issue where **bulk editing on HEAD** didn’t create the change set
  correctly.
- Fixed an issue where building the **deployment MV** failed if the previous
  version was missing.
- Fixed an issue where **multiple selected components** in a `toDelete` state
  were confusing in the UI.
- Fixed an issue where **components marked as toDelete on HEAD** leaked into
  change set reviews.
- Fixed an issue with using **duplicate component + default subscriptions**
  together.
- Fixed an issue where **abandoning a change set** via web app or Public API
  failed if the snapshot didn’t exist.

<iframe width="560" height="315" src="https://www.youtube.com/embed/TVzI8mKUmD0?si=y1wNNfP58cJeRF7B" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" referrerpolicy="strict-origin-when-cross-origin" allowfullscreen></iframe>

## September 22nd to 28th 2025

### ✨ Added

- Added more **user-facing documentation** in the onboarding flow.
- Users can now see the **code and arguments** for a queued action in the web
  app.
- Added **Azure CLI** back to the execution environment.
- Added extra **telemetry information** for Edda.

### 🔄 Changed

- Upgraded the **Axios dependency** in the MCP server.
- Implemented a better **MV rebuild system**.
- Upgraded the **Auth API** to TypeScript 5.x.
- Removed generation of **composable assets** from Clover.
- Cleaned up **ArgumentTargets** from the graph after connection removal.
- Cleaned up **parentage code** from the codebase after connection removal.
- Cleaned up **inferred connections** from the codebase after connection
  removal.

### 🐞 Fixed

- Fixed an issue in the **Auth Portal** where users weren’t redirected to their
  default workspace.
- Fixed an issue in the **Auth Portal** to prevent 404s when accessing the
  download link.
- Fixed an issue where a **function used in a subscription** failed to load in
  the UI.
- Fixed an issue in the **Auth API** to return the correct `workspaceId` for
  navigation.
- Fixed an issue where users could be **pushed to the lobby incorrectly** due to
  background activity.
- Fixed an issue where the **lobby exit event** waited for all processes to
  complete.
- Fixed an issue in the **Auth API Dockerfile** for TypeScript 5.x
  compatibility.
- Fixed an issue in **Public API** to show optional values correctly in API
  responses.
- Fixed an issue where the **App and Webworker** made too many requests.
- Fixed an issue where **create change set in Luminork** didn’t wait for MV to
  build.

<iframe width="560" height="315" src="https://www.youtube.com/embed/CnR3KL2OBxg?si=aU4t20K6bfBKy7kc" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" referrerpolicy="strict-origin-when-cross-origin" allowfullscreen></iframe>

## September 15th to 21st 2025

### ✨ Added

- Added support in **Public API** to access **component codegen data**.

### 🐞 Fixed

- Fixed a bug in **Public API** when creating authentication components.
- Fixed an issue in the **web app** where collaborator change set names were
  displayed incorrectly.

## September 8th to 14th 2025

### ✨ Added

- Added more **telemetry collection** to the onboarding flow.
- Enabled **review screen navigation** — back and forward buttons are now
  available if there are items to review.
- Split out **abandon, apply, and force_apply** MCP tools for finer control.

### 🔄 Changed

- Updated the **onboarding flow** for the web application.
- Simplified the **Public API** for creating qualifications and codegen
  functions.
- Compressed **deployment task requests** in Edda for efficiency.
- Updated Public API to allow **upgrading components by Schema Category** for
  more fine-grained control.
- Made additional **UX changes** to the onboarding flow.
- Removed all **sockets from Clover-based assets** and redeployed them.

### 🐞 Fixed

- Fixed an issue where **DVU status messages** incorrectly sent component
  updated notifications.
- Fixed a race condition in the **web worker**.
- Fixed an issue where users could get **stuck in the lobby** if the
  `checkOnboardingApi` call failed.
- Fixed an issue where **buttons changed size** during loading states.

<iframe width="560" height="315" src="https://www.youtube.com/embed/djEE7OGEVQI?si=Kn3Q-LbhkZ1karmA" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" referrerpolicy="strict-origin-when-cross-origin" allowfullscreen></iframe>

## September 2nd to 7th 2025

### ✨ Added

- Added **Component Details skeletons** for smoother loading.
- Added more **tracking to the Onboarding workflow**.
- Added **authoring endpoints** to Public API.
- Added a new **search endpoint for schemas** in the public API.
- Added support for **generating a template** to the MCP server.

### 🔄 Changed

- Updated to use the new **RBE binaries**.
- Updated **Nav bars** to use the v2 change set routes.
- Updated the **search components endpoint** in the public API to support
  filtering by _upgradable_.
- Released **v1.1.2 of the Public API SDKs**.
- Made it impossible to create **Socket Connections**.

### 🐞 Fixed

- Fixed a bug where leaving the **name blank on the secret form** could block
  submission.
- Fixed the **MCP server** to correctly send shutdown events.
- Fixed a race condition in **watchedForm** that could ignore user data
  mutations.
- Fixed the **abandon change set URL** in the new UX.
- Fixed an issue in the **Auth Portal** where users could be placed in a dev
  workspace instead of a production one.
- Fixed an issue where adding **secrets via Public API** was unnecessarily
  difficult.
- Fixed an issue where **multi-select showed the context menu** attached to the
  incorrect component.

<iframe width="560" height="315" src="https://www.youtube.com/embed/djEE7OGEVQI?si=sEWg2aRdZNLZKrtC" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" referrerpolicy="strict-origin-when-cross-origin" allowfullscreen></iframe>

## August 25th to 30th 2025

> **Note:** Public release of the new **Application platform**.

### ✨ Added

- Added **observability values** in the frontend for better monitoring.
- Added **toolbox helpers** to support ops work.
- Added more **metadata to pending event publish errors** for easier debugging.
- Added a **Public API endpoint** to record when a user has executed an AI agent
  at least once.
- Added a **flag in the UX** to warn users if they haven’t yet connected to the
  AI agent.

### 🔄 Changed

- Updated the **MCP Server** to use Public API as its source of schema
  attributes and documentation.
- Optimistically update the **Actions List on HEAD** for a faster UI experience.
- Terminate the **web worker** if the app is out of date, preventing users from
  being stuck in the lobby.
- Bypassed the **deployment MV index** for Frigg to ensure Public API stays
  current.
- Released **v1.0.9 of Public API SDKs**.
- Extended **index update payloads** with patch data.
- Moved the **create change set button** to use the v2 route.

### 🐞 Fixed

- Fixed **attribute value `before_value` events**.
- Fixed how **empty values** in DVU were being treated, which caused incorrect
  diffs.
- Fixed **flickering UX warning** caused by websocket reconnection.
- Removed the old **first-time modal** in Function Editor that was still firing
  incorrectly.

<iframe width="560" height="315" src="https://www.youtube.com/embed/T3QYwduCui4?si=r2AYvkHZdgMn9KAa" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" referrerpolicy="strict-origin-when-cross-origin" allowfullscreen></iframe>

## August 18th to 24th 2025

### ✨ Added

- Added **tags map** support to the property tree, preparing for future
  enhancements.
- Added a **note in change sets** so users know when they’re actively working
  inside one.
- Added the new **Subscriptions UI** (behind a feature flag).
- Added a **basic onboarding flow** (behind a feature flag).
- Added **empty states** for Default Subscriptions for a clearer experience.
- Added **audit logging** for default subscription create/delete events and
  attribute API calls.
- Added **erase, restore, and generate SI links tooling** to the MCP server.
- Added **PostHog tracking** to the MCP server.
- Added back the **debug screen** (behind a feature flag).

### 🔄 Changed

- Improved **system status messaging** for better clarity.
- Improved logic for running actions to reduce unnecessary rebaser trips.
- Updated **discovery and import management** functions.
- Expanded **Public API’s PropSpec** to include property documentation and more.
- Improved the **map regeneration experience**.
- Changed **multi-select and context menus** in the grid for easier navigation.
- Updated Public API endpoints (`get_schema`, `get_default_variant`) to use a
  new schema system.
- Removed the old **IAM permissions check**.
- Removed outdated **SDF routes**.
- Enhanced the **MCP server** to automatically clean up after itself.

### 🐞 Fixed

- Fixed issues with the **Apply Change Set button** (disabled incorrectly,
  showing at the wrong times, or flashing too quickly).
- Fixed problems in the **approvals flow** (missing buttons, modal not opening).
- Fixed crashes from **invalid connection states** in Pinga.
- Fixed overly restrictive **subscription blocking**.
- Fixed wording of **simulated change sets**.
- Fixed handling of **attribute updates** — errors now display to users.
- Fixed issues with the **Map and selected components**, including multi-select.
- Fixed styling and grouping in **Default Subscriptions**.
- Fixed issues where **data reactivity** wasn’t updating correctly.
- Fixed rendering on **resource values** and **bulk edit attribute screens**.
- Fixed actions not responding correctly when **running or dispatched**.
- Fixed missing **prop suggestions** when regenerating components.
- Fixed errors flushing **audit logs** in the rebaser.
- Fixed issues with **pending events** publishing.
- Fixed a bug in **Add Component modal** when double-clicking.
- Fixed a bug where the **new onboarding flow** could trap users in the lobby.
- Fixed broken **section headers** on the component details page.
- Fixed issues where the **MCP server could set incorrect attributes**.

<iframe width="560" height="315" src="https://www.youtube.com/embed/7E-a_BL59BM?si=u12NokBjHkFGivFD" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" referrerpolicy="strict-origin-when-cross-origin" allowfullscreen></iframe>

## August 11th to 17th 2025

### ✨ Added

- Added a **duplicate components endpoint** to Public API.
- Added ability to **save from the CodeEditor modal**.
- Added ability to **group by incompatible components** in a workspace.
- Added visibility into the **number of change sets being finalized** when the
  lobby is loading.
- Exposed the **component upgrade API** in Public API.
- Added **tracking to the Public API `whoami` call** (an MCP hotpath).
- Added support for **AWS IAM assets** in the MCP server.
- Added ability to **generate a template from Public API**.

### 🔄 Changed

- Made **Uninstalled Schema MVs** consumable via the web worker.
- Updated **Component Qualifications panel** styling and improved sorting of
  qualifications.

### 🐞 Fixed

- Fixed an issue in the **context menu** when navigating to the map.
- Fixed an issue where leaving **bulk edit mode** could leave the context menu
  floating at position `0,0`.
- Fixed an issue where users could be **sent to the lobby** on an `IndexUpdated`
  message.
- Fixed an issue with the **Component Secrets panel** and its validations.
- Fixed a race condition where **removeOldIndex** ran during cold start, causing
  foreign key issues in SQLite.
- Fixed handling of **MV Index `202` responses** by re-queuing change set
  requests.
- Fixed an issue where we were **suggesting the secret prop of all components**
  instead of the actual secret.
- Fixed an issue where the **browser used stale indexes**, serving outdated
  data.
- Fixed **404 errors** from Mjolnir on HEAD.
- Fixed an issue with the **search bar** causing filtered components to lose
  reactivity.

<iframe width="560" height="315" src="https://www.youtube.com/embed/cFdIUfTnDX0?si=NCRrAstvYdgnAO0M" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" referrerpolicy="strict-origin-when-cross-origin" allowfullscreen></iframe>

## August 4th to 10th 2025

### ✨ Added

- Added **import and discovery** capabilities to the MCP server.
- Added **workspace approvals** back into the new UI (still behind a feature
  flag).
- Added the ability to **set default subscriptions for arrays and maps** in the
  graph, and consume default subscriptions when creating a new component in the
  DAL.
- Added **history view** to the bulk edit page.

### 🔄 Changed

- Updated the **page layout** for bulk editing.
- Updated **Cypress tests** for the new UX.

### 🐞 Fixed

- Fixed an issue where the **Add Item button for arrays** refreshed instead of
  allowing additional entries.
- Fixed an issue where visiting a **component details page on HEAD** incorrectly
  created a new change set.
- Fixed an issue where the **Home page** was not redirecting correctly.
- Fixed an issue where **new workspaces** were not getting the correct token.
- Fixed an issue where running an **import function** didn’t trigger
  validations.
- Fixed an issue when adding a **key to a map** in the Attributes Panel.
- Fixed an issue where the **map failed** when encountering a missing edge.

<iframe width="560" height="315" src="https://www.youtube.com/embed/BMdpmyaSnEA?si=YTUD8vIW8scpB3yl" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" referrerpolicy="strict-origin-when-cross-origin" allowfullscreen></iframe>

## July 28th to August 3rd 2025

### ✨ Added

- Added ability to **run refresh action functions as part of change sets**,
  enhancing the import flow.
- Added ability to **specify a new name prefix** when duplicating components.
- Added the ability to **add and remove components from a view**.
- Added a new **SI MCP Server**.
- Added **sticky headers** to arrays and maps in the Attributes Panel.
- Added visibility into **how many components are filtered** out of a search.
- Added support for **secrets prop tree in bulk edit**.
- Exposed **Component Diff Status** on the grid tile (shows _Added_ and
  _Modified_ tags).
- Added a **region prop suggestion** to generated AWS-based templates.
- Added ability to **create DAL subscriptions** for scalar values.
- Added a **grid/map and component-level warning** when a workspace is using
  socket connections.

### 🔄 Changed

- Improved **management function detail output**.
- Enhanced the **right-click context menu**.
- Updated **keyboard shortcuts modal layout**.
- Ensured consistency across the **Public API and SDF** when passing attributes.
- Changed how the **visualize connections button** works — it now hides all
  unconnected items on the map.
- Applied more **consistency fixes for buttons**.
- Enabled the **new UX by default** for all users (behind a feature flag).
- Disabled the ability to **toggle back to the old UI** once feature-flagged
  into the new experience.

### 🐞 Fixed

- Fixed a bug when **restoring a component**.
- Fixed a bug where clicking an **action in the approval modal** led to a dead
  end.
- Fixed search queries not being up to date when **TanStack loaded from cache**.
- Fixed an issue where the **workspace token** wasn’t correct.
- Fixed an issue where **pin was incorrectly shown** in the map context menu.
- Fixed an issue where pasting a value into the **resourceId field** broke
  import.
- Fixed an issue where interacting with a **readonly attribute on HEAD** created
  a change set unnecessarily.
- Fixed a bug where **groupBy** logic was inverted.
- Fixed an issue where **links in the actions panel** weren’t working.
- Fixed an issue where **adding or removing a component from a view** didn’t
  redirect to the created change set.
- Fixed an issue with **bulk editing secrets**.
- Fixed an issue where we sometimes tried to **load a workspace with the wrong
  web token**.
- Fixed an issue where the **Add Component modal** was slow when typing.

<iframe width="560" height="315" src="https://www.youtube.com/embed/h089SVMlZgg?si=5f2kn3Y4NZSeBHJ8" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" referrerpolicy="strict-origin-when-cross-origin" allowfullscreen></iframe>

## July 21st to 27th 2025

### ✨ Added

- Added a **restyled component details page** for a refreshed look.
- Added the ability to **group by having a resource**.
- Added the ability to **re-run a qualification**.
- Exposed **RunTemplate** as a top-level component action.
- Added support to **set array and map items as subscriptions**.
- Introduced a new way to show **prop validation failures inline** with
  attributes.
- Added the ability to **take action on a component directly from the component
  details page**.
- Added the ability to **visualize component connections on the map**.
- Introduced the first pass at **bulk editing props**.
- Added **smooth pan to selected component** on the map.

### 🔄 Changed

- Improved **bulk insert** to speed up cold starts.
- Delivered more **performance improvements** when fetching data via Mjolnir.
- Retained **search filtering** after navigating away from a search.
- Changed the **component details page right-hand panel**.
- Debounced the **search function** to avoid multiple runs at once.
- Changed how **template-generated code output** is created and added the
  ability to create templates from the UI context menu.
- Redesigned the **component connections panel**.
- Ensured styles of the **component details, grid, and map** are consistent.
- Updated the **management function UX** for better usability.

### 🐞 Fixed

- Fixed the **search query** to correctly match what’s in the search bar.
- Fixed an issue where **regenerating a component** could lose prop-to-prop
  subscriptions.
- Fixed the **change set approval modal** not closing after merge to HEAD.
- Fixed an issue where the **migrate connections endpoint in SDF** wasn’t
  restricted to the admin API.
- Fixed issues to make **SQLite interactions** more robust.
- Fixed a bug where **multi-select actions** didn’t work on the map.
- Fixed a bug where **duplicating components** failed to reset prop
  subscriptions correctly.
- Fixed an issue to **preserve attribute order in templates**.

<iframe width="560" height="315" src="https://www.youtube.com/embed/gydyZw3cAfQ?si=piLzWpHiUf4j6zva" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" referrerpolicy="strict-origin-when-cross-origin" allowfullscreen></iframe>

## July 14th to 20th 2025

### ✨ Added

- Added ability in the web app to **save an attribute change on mouse blur**.

### 🔄 Changed

- Improved the **suggestions experience** in the web app.
- Updated **CodeMirror themes** in the web app to GitHub Dark and Light.
- Removed **sync recursion** on `AttributeValue::view` in DAL.

### 🐞 Fixed

- Fixed an issue in the web app where the **actions list** was not displayed
  correctly.
- Fixed an issue in the web app where **skeletons for the map** were not shown
  properly.
- Fixed an issue in the web app to correctly display when an **array is a
  subscription** and prevent changes to it.

<iframe width="560" height="315" src="https://www.youtube.com/embed/vxMZ9DW9EH4?si=EgjBbpZQbwNmlnhR" title="YouTube video player" frameborder="0" allow="accelerometer; autoplay; clipboard-write; encrypted-media; gyroscope; picture-in-picture; web-share" referrerpolicy="strict-origin-when-cross-origin" allowfullscreen></iframe>

## Previous changes

All old changes are tracked in our weekly demo videos on our
[YouTube channel](https://www.youtube.com/@systeminit/videos)
