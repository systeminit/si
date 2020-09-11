use si_data::{DataError, Db};

use crate::error::Result;
pub use crate::protobuf::{
    BillingAccount, BillingAccountSignupReply, BillingAccountSignupRequest, Capability, Group,
    GroupSiProperties, Integration, IntegrationInstance, IntegrationInstanceSiProperties,
    Organization, OrganizationSiProperties, User, UserSiProperties, Workspace,
    WorkspaceSiProperties,
};
use tracing::debug;

impl BillingAccount {
    pub async fn signup(
        db: &Db,
        request: BillingAccountSignupRequest,
    ) -> Result<BillingAccountSignupReply> {
        debug!("billing account req");
        let billing_account_req = request
            .billing_account
            .ok_or_else(|| DataError::RequiredField("billingAccount".into()))?;

        debug!("billing account create");
        let billing_account = BillingAccount::create(
            db,
            billing_account_req.name,
            billing_account_req.display_name,
        )
        .await?;

        debug!("user");
        let user_req = request
            .user
            .ok_or_else(|| DataError::RequiredField("user".into()))?;

        debug!("user si properties");
        let user_si_properties = UserSiProperties {
            billing_account_id: billing_account.id.clone(),
        };

        debug!("user create");
        let user = User::create(
            db,
            user_req.name,
            user_req.display_name,
            user_req.email,
            user_req.password,
            Some(user_si_properties),
        )
        .await?;

        debug!("admin group si properties");
        let group_si_properties = GroupSiProperties {
            billing_account_id: billing_account.id.clone(),
        };

        debug!("admin group capabilities");
        let group_capabilities = vec![Capability {
            subject: billing_account.id.clone(),
            actions: vec!["any".to_string()],
        }];

        debug!("admin group create");
        let _group = Group::create(
            db,
            Some("administrators".to_string()),
            Some("Administrators".to_string()),
            vec![user.id.as_ref().unwrap().clone()],
            Some(group_si_properties),
            group_capabilities,
        )
        .await?;

        debug!("organization si properties");
        let organization_si_properties = OrganizationSiProperties {
            billing_account_id: billing_account.id.clone(),
        };

        debug!("organization create");
        let organization = Organization::create(
            db,
            Some("default".to_string()),
            Some("Default".to_string()),
            Some(organization_si_properties),
        )
        .await?;

        debug!("workspace si properties");
        let workspace_si_properties = WorkspaceSiProperties {
            billing_account_id: billing_account.id.clone(),
            organization_id: organization.id.clone(),
        };

        debug!("workspace create");
        let workspace = Workspace::create(
            db,
            Some("default".to_string()),
            Some("Default".to_string()),
            Some(workspace_si_properties),
        )
        .await?;

        debug!("get global integration");
        let global_integration =
            Integration::get_by_natural_key(db, "global:integration:global").await?;

        debug!("create global integration instance");
        let _integration_instance = IntegrationInstance::create(
            db,
            Some("global".to_string()),
            Some("Global Integration".to_string()),
            vec![],
            Some(IntegrationInstanceSiProperties {
                billing_account_id: billing_account.id.clone(),
                integration_id: global_integration.id.clone(),
                enabled_workspace_id_list: vec![workspace.id.as_ref().unwrap().clone()],
                enabled_organization_id_list: vec![organization.id.as_ref().unwrap().clone()],
            }),
        )
        .await?;

        debug!("get aws integration");
        let aws_integration = Integration::get_by_natural_key(db, "global:integration:aws").await?;

        debug!("create aws integration instance");
        let _aws_integration_instance = IntegrationInstance::create(
            db,
            Some("aws".to_string()),
            Some("AWS Integration".to_string()),
            vec![],
            Some(IntegrationInstanceSiProperties {
                billing_account_id: billing_account.id.clone(),
                integration_id: aws_integration.id.clone(),
                enabled_workspace_id_list: vec![workspace.id.as_ref().unwrap().clone()],
                enabled_organization_id_list: vec![organization.id.as_ref().unwrap().clone()],
            }),
        )
        .await?;

        debug!("sending reply");
        Ok(BillingAccountSignupReply {
            billing_account: Some(billing_account),
            user: Some(user),
            organization: Some(organization),
            workspace: Some(workspace),
        })
    }
}
