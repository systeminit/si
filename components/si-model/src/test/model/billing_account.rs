use names::{Generator, Name};
use si_data::{NatsConn, NatsTxn, PgPool, PgTxn};
use crate::{BillingAccount, Group, Organization, PublicKey, User, Veritech, Workspace};

#[derive(Debug)]
pub struct NewBillingAccount {
    pub billing_account: BillingAccount,
    pub user: User,
    pub group: Group,
    pub workspace: Workspace,
    pub organization: Organization,
    pub public_key: PublicKey,
    pub user_password: String,
    //pub system: System,
}

pub async fn create_new_billing_account(txn: &PgTxn<'_>, nats: &NatsTxn) -> BillingAccount {
    let mut generator = Generator::with_naming(Name::Numbered);
    let name = generator.next().unwrap();
    BillingAccount::new(&txn, &nats, &name, format!("{} description", name))
        .await
        .expect("cannot create billing account")
}

pub async fn signup_new_billing_account(
    pg: &PgPool,
    _txn: &PgTxn<'_>,
    nats: &NatsTxn,
    nats_conn: &NatsConn,
    veritech: &Veritech,
) -> NewBillingAccount {
    let mut generator = Generator::with_naming(Name::Numbered);
    let billing_account_name = generator.next().unwrap();
    let mut name_generator = Generator::default();
    let user_name = name_generator.next().unwrap();
    let user_password = name_generator.next().unwrap();
    let mut nba_conn = pg.get().await.expect("cannot get connection");
    let nba_txn = nba_conn
        .transaction()
        .await
        .expect("cannot open new transaction");
    let (billing_account, user, group, organization, workspace, public_key) =
        BillingAccount::signup(
            &pg,
            nba_txn,
            &nats,
            &nats_conn,
            &veritech,
            &billing_account_name,
            format!("{} description", billing_account_name),
            &user_name,
            format!("{}@tclown.com", user_name),
            &user_password,
        )
        .await
        .expect("cannot create new billing account");
    NewBillingAccount {
        billing_account,
        user,
        group,
        organization,
        workspace,
        public_key,
        user_password,
    }
}
