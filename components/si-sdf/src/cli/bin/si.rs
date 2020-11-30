extern crate si_sdf;

use anyhow::Result;

use si_sdf::cli::client::{ChangeRun, Client, Command, DebugFormatter};

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = Client::new("ws://localhost:5156/cli?token=Bearer+eyJhbGciOiJSUzI1NiIsImtpZCI6Imp3dEtleTpiYjc4MmIzOTQ5MTM0NGRiODVmMmE0N2JiNmUzNzgwMiIsInR5cCI6IkpXVCJ9.eyJpYXQiOjE2MDYzMzQ2ODEsImV4cCI6MTYwNjQyMTA4MSwibmJmIjoxNjA2MzM0NjgxLCJpc3MiOiJodHRwczovL2FwcC5zeXN0ZW1pbml0LmNvbSIsInN1YiI6InVzZXI6MDQwZWNmNDcwYmQzNGI4NDg2NWE4M2YzNzI4Yzg4YzAiLCJhdWQiOiJodHRwczovL2FwcC5zeXN0ZW1pbml0LmNvbSIsInVzZXJJZCI6InVzZXI6MDQwZWNmNDcwYmQzNGI4NDg2NWE4M2YzNzI4Yzg4YzAiLCJiaWxsaW5nQWNjb3VudElkIjoiYmlsbGluZ0FjY291bnQ6ZTI2YzdkMTIzODRlNDBjYmE4ODk3NTE4MmIwMDE1NDEifQ.fiwryVCsi0oqCwO71_3H_g60VkdA2UOIMXKq_84Wh6Bjibk7_j2p0W5s3pgpXm7miiLAecIoyr7SGfOXc_UslLRG0AFsz_92rmKHYE5FK177MYMYTRItglVms-ktWU5M6bscouG8BN2u67t9vLhZN7CFCdPdGQ2PLKFzpxfP6gFTGyKzFFbu1oNkEobTGaPhOtUsc-xCJUTKCt3ucKYphPrdeq9Q0HrIpVBnN_ysgo9OXrkEqvBfiDcSc9i0CXMZzFvf7rFn16Zgbx5R0KOLiBBZQgY1JkLPnkE7h17XTYLon8eJdp4u3CKnyTclPiKIYJDK7a54vnxDw7KNBS5Ysg", DebugFormatter::new());

    let change_run = ChangeRun::new(
        "entity:eb12bcaa15414dd7a2df01270e332efb",
        "system:fc5ab1d6de74433c9705b45db6a04568",
        "deploy",
    );
    let command = Command::ChangeRun(change_run);
    client.command(command).await?;
    Ok(())
}
