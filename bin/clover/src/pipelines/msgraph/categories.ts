/**
 * Service category mapping for Microsoft Graph resources.
 * Maps resource names to Microsoft 365 service categories.
 *
 * Resources are named like Azure ARM: Microsoft.{Service}/{Resource}
 * Example: Microsoft.Entra/users, Microsoft.Teams/teams
 */

// TODO: it would be better to create a heurisitc for this, but one was not
// readily apparent in the spec. If we find new services being added to this
// spec, we can revisit this.
export const GRAPH_SERVICE_CATEGORIES: Record<string, string> = {
  // Microsoft Entra (Identity & Directory Services)
  "users": "Entra",
  "groups": "Entra",
  "groupLifecyclePolicies": "Entra",
  "groupSettings": "Entra",
  "groupSettingTemplates": "Entra",
  "applications": "Entra",
  "servicePrincipals": "Entra",
  "directory": "Entra",
  "directoryRoles": "Entra",
  "directoryRoleTemplates": "Entra",
  "directoryObjects": "Entra",
  "identity": "Entra",
  "identityGovernance": "Entra",
  "identityProtection": "Entra",
  "identityProviders": "Entra",
  "domains": "Entra",
  "domainDnsRecords": "Entra",
  "oauth2PermissionGrants": "Entra",
  "permissionGrants": "Entra",
  "invitations": "Entra",
  "contacts": "Entra",
  "contracts": "Entra",
  "agreements": "Entra",
  "agreementAcceptances": "Entra",
  "certificateBasedAuthConfiguration": "Entra",
  "authenticationMethodConfigurations": "Entra",
  "authenticationMethodsPolicy": "Entra",
  "schemaExtensions": "Entra",
  "scopedRoleMemberships": "Entra",
  "me": "Entra",
  "organization": "Entra",
  "subscribedSkus": "Entra",

  // Microsoft Teams
  "teams": "Teams",
  "teamsTemplates": "Teams",
  "teamwork": "Teams",
  "chats": "Teams",
  "communications": "Teams",

  // SharePoint & OneDrive
  "sites": "SharePoint",
  "drives": "Files",
  "shares": "Files",

  // Intune / Device Management
  "deviceManagement": "Intune",
  "deviceAppManagement": "Intune",
  "devices": "Intune",

  // Security & Compliance
  "security": "Security",
  "compliance": "Compliance",
  "auditLogs": "Security",
  "informationProtection": "Security",
  "privacy": "Compliance",

  // Administration & Governance
  "admin": "Admin",
  "reports": "Reporting",
  "policies": "Governance",
  "roleManagement": "Governance",
  "tenantRelationships": "Admin",
  "dataPolicyOperations": "Governance",

  // Collaboration & Productivity
  "planner": "Planner",
  "education": "Education",
  "employeeExperience": "Collaboration",
  "places": "Collaboration",
  "copilot": "Copilot",
  "solutions": "Solutions",

  // Other Services
  "appCatalogs": "Apps",
  "applicationTemplates": "Apps",
  "search": "Search",
  "connections": "Search",
  "external": "Search",
  "print": "UniversalPrint",
  "storage": "Storage",
  "subscriptions": "Webhooks",
  "functions": "Utility",
  "filterOperators": "Utility",
};

export function getGraphServiceCategory(resourceName: string): string {
  return GRAPH_SERVICE_CATEGORIES[resourceName] || resourceName;
}
