
use std::collections::HashMap;
use super::SessionError;
use super::SessionResult;
use crate::server::extract::{HandlerContext, JwtSecretKey};
use axum::Json;
use dal::UserPk;
use dal::Workspace;
use dal::WorkspacePk;
use dal::{context::AccessBuilder, HistoryActor, StandardModel, Tenancy, User};
use serde::{Deserialize, Serialize};
use serde_json::json;


#[derive(Debug, Serialize, Deserialize, Clone, )]
#[serde(rename_all = "camelCase")]
pub struct AuthConnectRequest {
	pub code: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthConnectResponse {
	pub user: User,
	pub workspace: Workspace,
	pub token: String,
}

#[derive(Debug, Serialize, Deserialize)]
struct AuthApiErrBody {
	pub kind: String,
	pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthApiUser {
	// probably dont really care about anything here but the id
	// but we may want to cache name and email? TBD...
	pub id: UserPk,
	pub nickname: String,
	pub first_name: String,
	pub last_name: String,
	pub picture_url: Option<String>,
	pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthApiWorkspace {
	pub id: WorkspacePk,
	pub slug: String,
	pub display_name: String,
	// probably some more data here...
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AuthApiConnectResponse {
	pub user: AuthApiUser,
	pub workspace: AuthApiWorkspace,
	pub token: String,
}


pub async fn auth_connect(
	HandlerContext(builder): HandlerContext,
	JwtSecretKey(jwt_secret_key): JwtSecretKey,
	Json(request): Json<AuthConnectRequest>,
) -> SessionResult<Json<AuthConnectResponse>> {

	let client = reqwest::Client::new();
	let res = client.post("http://localhost:9001/complete-auth-connect")
		.json(&json!({"code": request.code }))
		.send()
		.await?;

	if res.status() != reqwest::StatusCode::OK {
		let res_err_body = res.json::<AuthApiErrBody>()
			.await
			.map_err(|err| SessionError::AuthApiError(err.to_string()))?;
		println!("code exchange failed = {:?}", res_err_body.message);
		return Err(SessionError::AuthApiError(res_err_body.message));
	}


	// println!("body text = {:?}", res.text().await);

	let res_body = res.json::<AuthApiConnectResponse>()
		.await?;
		// .map_err(|err| SessionError::AuthApiError(err.to_string()))?;
	println!("body = {:?}", res_body);	

		
	let mut ctx = builder
		.build(
				AccessBuilder::new(
						// Empty tenancy means things can be written, but won't ever be read by whatever uses the standard model
						Tenancy::new_empty(),
						HistoryActor::SystemInit,
				)
				.build_head(),
		)
		.await?;


	// lookup user or create if we've never seen it before
	let maybe_user = User::get_by_pk(&ctx, res_body.user.id).await?;
	let user = match maybe_user {
		Some(user) => user,
		None => User::new(&ctx, res_body.user.nickname, res_body.user.email).await?,
	};

	// lookup workspace or create if we've never seen it before
	let maybe_workspace = Workspace::get_by_pk(&ctx, &res_body.workspace.id).await?;
	let workspace = match maybe_workspace {
		Some(workspace) => {
			ctx.update_tenancy(Tenancy::new(*workspace.pk()));
			workspace
		},
		None => Workspace::new(&mut ctx, res_body.workspace.display_name).await?,
	};

	// ensure workspace is associated to user
	user.associate_workspace(&ctx, *workspace.pk()).await?;

	Ok(Json(AuthConnectResponse {
		user,
		workspace,
		token: res_body.token
	}))
}
