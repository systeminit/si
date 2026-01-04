---
outline: [2, 3]
---

# Policy Layers in System Initiative

System Initiative is an AI Native Infrastructure Automation Platform that builds [digital twins](https://docs.systeminit.com/explanation/architecture/digital-twin) of your infrastructure. It enables you to define and enforce policy across three distinct layers, each serving different governance needs.

## Overview

System Initiative allows you to write policy in three complementary ways, from preventive controls to detective controls to compliance verification:

### 1. Native Cloud Provider Policy

**Description**: Model and enforce the native policy mechanisms available in your cloud provider.

**Examples**:
- AWS Resource Control Policies (RCP)
- AWS Service Control Policies (SCP)
- Azure Policy

**Characteristics**:
- **Preventive**: Blocks non-compliant resources from being created
- **Hard enforcement**: Resources cannot be deployed if they violate policy
- **Provider-native**: Uses the cloud provider's built-in policy engine
- **Centralized control**: Typically applied at the organization or account level

**Use case**: When you need to prevent specific actions or resource configurations from ever being created (e.g., "no S3 buckets in eu-west-1" or "all EC2 instances must use approved AMIs").

### 2. Component-Level Qualifications

**Description**: Define requirements and best practices for individual component types within System Initiative.

**Examples**:
- Ensure all `AWS::S3::Bucket` resources have encryption enabled
- Verify all `AWS::EC2::Instance` resources use specific instance types
- Check that `AWS::RDS::DBInstance` has backup retention configured

**Characteristics**:
- **Detective**: Flags non-compliant configurations but doesn't prevent creation
- **Soft enforcement**: Allows operators to make informed decisions about remediation
- **Component-specific**: Applied to specific resource types
- **Operator-driven resolution**: You decide when and how to reconcile flagged issues

**Use case**: When you want to maintain best practices and standards but need flexibility for operators to handle exceptions or special cases. The system alerts you to potential issues, but you maintain control over the response.

### 3. Control Document Evaluation with AI Agents

**Description**: Write high-level control documents that describe your compliance requirements, then use AI agents to evaluate your infrastructure against those controls.

**Examples**:
- "All data at rest must be encrypted"
- "No public internet access to databases"
- "Logging must be enabled for all audit-relevant services"
- "Multi-factor authentication required for privileged access"

**Characteristics**:
- **Compliance-focused**: Maps to regulatory frameworks (SOC 2, HIPAA, PCI-DSS, etc.)
- **AI-powered evaluation**: Agents interpret control language and assess infrastructure
- **Detailed reporting**: Provides comprehensive reports on control adherence
- **Natural language**: Write controls in plain English rather than code

**Use case**: When you need to demonstrate compliance with regulatory requirements or internal security standards. The AI agent evaluates your actual deployed infrastructure and generates reports showing which controls are met and which require attention.

## How the Layers Work Together

These three policy layers complement each other to provide defense in depth:

1. **Native cloud policies** prevent the worst violations from ever occurring
2. **Component qualifications** catch configuration drift and suboptimal patterns during design
3. **Control document evaluation** provides ongoing compliance verification and audit reporting

By combining all three layers, System Initiative enables you to build secure, compliant infrastructure while maintaining the flexibility needed for operational efficiency.
