# si-conduit

A command-line tool for authoring System Initiative schemas locally and pushing them to your workspaces.

## Architecture

SI Conduit is a Deno-based CLI application that provides a structured workflow for managing System Initiative schemas. The architecture consists of several key components:

### Core Components

- **CLI Module** (`src/cli.ts`): Command-line interface built with Cliffy, providing a hierarchical command structure with global options and environment variable support.
- **Context Module** (`src/context.ts`): Singleton context managing global application state, including logging (LogTape) and analytics (PostHog) services.
- **Project Module** (`src/project.ts`): Project structure management and path utilities for working with schemas and their functions.
- **Authentication** (`src/auth-api-client.ts`, `src/jwt.ts`): API authentication and JWT token handling for secure communication with System Initiative services.
- **Generators** (`src/generators.ts`): Code generation utilities for scaffolding schemas and functions.

### Command Structure

```
si-conduit
├── schema
│   ├── action generate      - Generate action functions (create, destroy, refresh, update)
│   ├── codegen generate     - Generate code generator functions
│   ├── management generate  - Generate management functions
│   ├── qualification generate - Generate qualification functions
│   └── scaffold generate    - Scaffold a complete schema structure
├── remote
│   └── push                 - Push schemas to remote workspace
└── whoami                   - Display authenticated user information
```

### Project Structure

SI Conduit projects follow this directory structure:

```
project-root/
├── .conduitroot             - Marker file identifying the project root
└── schemas/
    └── <schema-name>/
        ├── .format-version
        ├── schema.ts
        ├── schema.metadata.json
        ├── actions/
        │   ├── create.ts
        │   ├── create.metadata.json
        │   ├── destroy.ts
        │   ├── destroy.metadata.json
        │   ├── refresh.ts
        │   ├── refresh.metadata.json
        │   ├── update.ts
        │   └── update.metadata.json
        ├── codeGenerators/
        │   ├── <codegen-name>.ts
        │   └── <codegen-name>.metadata.json
        ├── management/
        │   ├── <management-name>.ts
        │   └── <management-name>.metadata.json
        └── qualifications/
            ├── <qualification-name>.ts
            └── <qualification-name>.metadata.json
```

### Code Quality

The project enforces code quality through:

- **Custom Lint Rules**: Prohibits direct usage of `Deno.env.get()` to ensure proper configuration management through the Context singleton.
- **TypeScript Strict Mode**: Type-safe path handling with specialized path classes (AbsolutePath, RelativePath).
- **Structured Logging**: LogTape integration with configurable verbosity levels.

## Configuration

### Environment Variables

- `SI_API_TOKEN`: Your System Initiative API token (required for authenticated commands)
- `SI_API_BASE_URL`: API endpoint URL (defaults to `https://api.systeminit.com`)
- `SI_CONDUIT_ROOT`: Project root directory (searches for `.conduitroot` if not specified)

### Global Options

All commands support these options:

- `--api-token <TOKEN>`: API authentication token
- `--api-base-url <URL>`: Override the API endpoint
- `--root <PATH>`: Specify project root directory
- `-v, --verbose [level]`: Enable verbose logging (0=errors only, 1=+warnings, 2=+info, 3=+debug, 4=+trace)
- `--no-color`: Disable colored output

## Development

### Prerequisites

- [Deno](https://deno.land/) runtime (version 1.40+)
- For Buck2 builds: [Buck2](https://buck2.build/) build system

### Running in Development Mode

Run the CLI in development mode with hot reloading:

```bash
deno task dev
```

Without arguments, this displays the help text listing all available commands.

### Running Specific Commands

```bash
# Display help
deno task dev --help

# Generate a schema scaffold
deno task dev schema scaffold generate MySchema

# Push schemas to remote workspace
deno task dev remote push
```

## Building

### Building with Deno

Build a standalone executable in the current directory:

```bash
deno task build
```

This creates the `si-conduit` executable with all necessary permissions.

### Building with Buck2

Build using the Buck2 build system:

```bash
buck2 build bin/si-conduit
```

The compiled binary will be located in the Buck2 output directory.

For production builds:

```bash
buck2 build //bin/si-conduit --mode=release
```

## Testing and Code Quality

### Running Tests

```bash
deno task test
```

### Linting

```bash
deno task lint
```

This project uses custom lint rules to enforce code quality. Notably, direct usage of `Deno.env.get()` is prohibited to ensure proper configuration management.

### Formatting

Check code formatting:

```bash
buck2 run //bin/si-conduit:check-format
```

Auto-fix formatting issues:

```bash
buck2 run //bin/si-conduit:fix-format
```

## Installation

### Remote Installation (Recommended)

Build and install directly from GitHub without cloning the repository:

```bash
deno compile \
  --allow-all \
  --reload \
  --output=si-conduit \
  --import-map=https://raw.githubusercontent.com/systeminit/si/main/bin/si-conduit/deno.json \
  https://raw.githubusercontent.com/systeminit/si/main/bin/si-conduit/main.ts
```

This downloads the source, compiles it, and creates the `si-conduit` executable in the current directory.

For a specific version or branch, replace `main` with the desired Git reference:

```bash
deno compile \
  --allow-all \
  --reload \
  --output=si-conduit \
  --import-map=https://raw.githubusercontent.com/systeminit/si/v1.0.0/bin/si-conduit/deno.json \
  https://raw.githubusercontent.com/systeminit/si/v1.0.0/bin/si-conduit/main.ts
```

### Local Installation

After building locally (see [Building](#building)), move the executable to a directory in your PATH:

```bash
# Build locally
deno task build

# Install to user bin directory (Linux/macOS)
mv si-conduit ~/.local/bin/

# Or install system-wide (requires sudo)
sudo mv si-conduit /usr/local/bin/
```

## Usage

### Initializing a Project

Create a `.conduitroot` marker file in your project root:

```bash
touch .conduitroot
```

The CLI will search for this file when determining the project root.

### Creating a Schema

Generate a complete schema scaffold:

```bash
si-conduit schema scaffold generate MySchema
```

This creates the schema directory structure with template files for the schema definition and metadata.

### Generating Functions

Generate specific function types for a schema:

```bash
# Generate an action function
si-conduit schema action generate MySchema create

# Generate a code generator
si-conduit schema codegen generate MySchema terraform

# Generate a management function
si-conduit schema management generate MySchema reconcile

# Generate a qualification
si-conduit schema qualification generate MySchema validate
```

### Pushing to Remote

Push your schemas to your System Initiative workspace:

```bash
si-conduit remote push
```

This command requires authentication via the `SI_API_TOKEN` environment variable.

### Checking Authentication

Verify your authentication status:

```bash
si-conduit whoami
```

## Troubleshooting

### Authentication Issues

If you encounter authentication errors:

1. Verify your `SI_API_TOKEN` is set correctly:
   ```bash
   echo $SI_API_TOKEN
   ```

2. Check token validity:
   ```bash
   si-conduit whoami
   ```

3. Ensure you're using the correct API endpoint with `--api-base-url` if needed.

### Project Root Not Found

If the CLI cannot find your project root:

1. Verify `.conduitroot` exists in your project root directory
2. Use the `--root` flag to explicitly specify the project root:
   ```bash
   si-conduit --root /path/to/project schema scaffold generate MySchema
   ```

### Verbose Logging

Enable verbose logging to debug issues:

```bash
# Maximum verbosity (trace level)
si-conduit -vvvv schema scaffold generate MySchema

# Or specify a numeric level (0-4)
si-conduit --verbose 4 schema scaffold generate MySchema
```

## Contributing

### Development Tasks

- [x] Read format-version on push
- [x] Use asset name from metadata.json
- [x] Qualifications handling
- [x] Push management functions
- [x] Push code generators
- [x] Add PostHog analytics events
- [x] Write comprehensive README
- [ ] Handle existing function bindings for asset updates
- [ ] Handle existing function names for new and updating assets

### Code Style

- Follow the existing code structure and TypeScript conventions
- Use the Context singleton for logging and analytics
- Leverage the Project module for path management
- Add JSDoc comments for public APIs
- Run tests and linting before submitting changes

## License

See the main System Initiative repository for license information.

