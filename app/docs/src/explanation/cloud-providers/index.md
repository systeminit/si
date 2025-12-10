# Cloud Providers

System Initiative connects to cloud providers to manage your infrastructure. Each provider integrates differently based on their APIs and authentication methods.

## Supported Providers

System Initiative currently supports the following cloud providers:

- [AWS (Amazon Web Services)](./aws.md) - Full support with multiple authentication methods
- [Azure (Microsoft Azure)](./azure.md) - Beta support with Service Principal authentication
- [DigitalOcean](./digital-ocean.md) - Beta support with API token authentication
- [Hetzner Cloud](./hetzner.md) - Beta support with API token authentication

## How Provider Integration Works

System Initiative connects to cloud providers through their official APIs:

- **API Integration**: Each provider uses its native API (AWS Cloud Control API, Azure ARM REST API, or OpenAPI specifications)
- **Authentication**: Credentials are managed through secure credential components in your workspace
- **Discovery**: You can discover existing infrastructure from your cloud accounts
- **Management**: Create, update, and delete resources through System Initiative's unified interface

## Request Additional Provider Support

Need support for a provider not listed here? [Request it on our Discord](https://discord.com/invite/system-init) or [open an issue on GitHub](https://github.com/systeminit/si/issues).
