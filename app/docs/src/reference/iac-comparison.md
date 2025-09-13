# Infrastructure as Code vs System Initiative

Everything that can be accomplished with traditional Infrastructure as Code (IaC) tooling can be done using System Initiative. This reference highlights key differences in methodology, workflow, and general capabilities. Learn more about [System Initiative's architecture](./architecture/index.md).

## Comparison

| Aspect | Traditional IaC (Terraform) | System Initiative |
|--------|----------------------------|-------------------|
| **Initialization** | `terraform init` - Initialize working directory with provider plugins and backend configuration | Creating a workspace and adding credentials through the visual interface, API or AI integration |
| **State Management** | External state storage (local files, S3/DynamoDB, GCS, Terraform Cloud) with manual configuration | Centralized state store embedded in the system with bi-directional data model and automatic synchronization |
| **State Locking** | External locking mechanisms (DynamoDB, Terraform Enterprise) to prevent concurrent modifications | Built-in Change Sets with automatic rebasing of merges to HEAD, eliminating traditional locking concerns |
| **Secrets Management** | Integration with external secret stores (Vault, AWS SSM, Secrets Manager) using data sources | Fully encrypted secrets management built into the platform with end-to-end encryption in transit and at rest. Highly extensible code-first platform allowing any interfacing secret backend to be supported |
| **Drift Detection** | `terraform plan` to detect differences between desired state and actual infrastructure, limited to managed resources | Full-fidelity discovery and import with bidirectional diff capabilities across all infrastructure |
| **Resource Removal** | `terraform state rm` and manual code removal to stop managing resources | Component erase from the graph with clear relationship impact visualization |
| **Manual Overrides** | `terraform taint` to force resource recreation on next apply | Direct action queuing (Delete + Create) against specific components with immediate contextual feedback |
| **Data Flow** | Output blocks and data sources for passing values between modules and external data retrieval | System and user-authored functions for data transformation and binding between components with real-time updates |
| **Version Management** | `required_providers` blocks and `terraform.lock.hcl` for provider/module version pinning | On-demand asset upgrades with granular component-level version control |
| **Branching Strategy** | Git branches for parallel development with manual merge conTask | Change Sets as automatically rebasing branches with conflict-free merges |
| **Change Review** | Pull requests with external CI/CD integration for plan/apply workflows | Built-in change review system with granular approvals and real-time collaboration |
| **Resource Import** | `terraform import` with manual state file manipulation and code generation | Full-fidelity discovery and import with automatic relationship detection and visual integration |
| **Selective Apply** | `terraform apply -target` for applying subset of Changes | Action queuing system allowing selective execution of specific actions or action groups |
| **Environment Management** | Terraform workspaces or directory/repository cloning per environment | Templates and duplication functions with workspace, view, or RBAC level environment separation |
| **Policy as Code** | External tools (Sentinel, OPA, Checkov) integrated via CI/CD or pre-commit hooks | Native qualifications system integrated directly into Change Sets with real-time validation |
| **Visualization** | Static `terraform graph` output requiring external tools for visualization | Native dynamic graph visualization with interactive Map view and real-time updates |
| **CI/CD Integration** | GitHub Actions, Jenkins, or similar pipelines for plan-on-PR and apply-on-merge workflows | Deep external API interface for integration with any CI provider or custom scripting |
| **Reusability** | Terraform modules for packaging and sharing reusable infrastructure patterns | Templates system for creating and sharing reusable infrastructure patterns with visual composition |

## Key Differences

### From Files to Data

Traditional IaC manages infrastructure through disparate text files and state files, while System Initiative models infrastructure as a unified data model where relationships between declarations and configuration are explicit as opposed to inferred.

### From Sequential to Real-time

While traditional IaC follows a model-plan-apply cycle, System Initiative provides real-time updates and validation as you build and modify your infrastructure model.

### From External Dependencies to Integrated Platform

Traditional IaC requires orchestration of multiple external tools for state management, secrets, CI/CD, policy, and visualization. System Initiative provides an integrated platform with these capabilities built in.
