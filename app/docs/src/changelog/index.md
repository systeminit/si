# Changelog

All notable changes to System Initiative will be documented in this file.

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
- Updated Luminork to allow **upgrading components by Schema Category** for more
  fine-grained control.
- Made additional **UX changes** to the onboarding flow.
- Removed all **sockets from Clover-based assets** and redeployed them.

### 🐞 Fixed

- Fixed an issue where **DVU status messages** incorrectly sent component
  updated notifications.
- Fixed a race condition in the **web worker**.
- Fixed an issue where users could get **stuck in the lobby** if the
  `checkOnboardingApi` call failed.
- Fixed an issue where **buttons changed size** during loading states.

## September 2nd to 7th 2025

### ✨ Added

- Added **Component Details skeletons** for smoother loading.
- Added more **tracking to the Onboarding workflow**.
- Added **authoring endpoints** to Luminork.
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
- Fixed an issue where adding **secrets via Luminork** was unnecessarily
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
- Added a **Luminork endpoint** to record when a user has executed an AI agent
  at least once.
- Added a **flag in the UX** to warn users if they haven’t yet connected to the
  AI agent.

### 🔄 Changed

- Updated the **MCP Server** to use Luminork as its source of schema attributes
  and documentation.
- Optimistically update the **Actions List on HEAD** for a faster UI experience.
- Terminate the **web worker** if the app is out of date, preventing users from
  being stuck in the lobby.
- Bypassed the **deployment MV index** for Frigg to ensure Luminork stays
  current.
- Released **v1.0.9 of Luminork SDKs**.
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
- Expanded **Luminork’s PropSpec** to include property documentation and more.
- Improved the **map regeneration experience**.
- Changed **multi-select and context menus** in the grid for easier navigation.
- Updated Luminork endpoints (`get_schema`, `get_default_variant`) to use a new
  schema system.
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

- Added a **duplicate components endpoint** to Luminork.
- Added ability to **save from the CodeEditor modal**.
- Added ability to **group by incompatible components** in a workspace.
- Added visibility into the **number of change sets being finalized** when the
  lobby is loading.
- Exposed the **component upgrade API** in Luminork.
- Added **tracking to the Luminork `whoami` call** (an MCP hotpath).
- Added support for **AWS IAM assets** in the MCP server.
- Added ability to **generate a template from Luminork**.

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
- Ensured consistency across the **Luminork API and SDF** when passing
  attributes.
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
