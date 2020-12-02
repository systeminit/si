use anyhow::Result;

use si_sdf::cli::client::{ChangeRun, Client, Command, DebugFormatter};

#[tokio::main]
async fn main() -> Result<()> {
    let mut client = Client::new("ws://localhost:5156/cli?token=Bearer+eyJhbGciOiJSUzI1NiIsImtpZCI6Imp3dEtleTpiYjc4MmIzOTQ5MTM0NGRiODVmMmE0N2JiNmUzNzgwMiIsInR5cCI6IkpXVCJ9.eyJpYXQiOjE2MDY4NjgwODIsImV4cCI6MjIzNzU4ODA4MiwibmJmIjoxNjA2ODY4MDgyLCJpc3MiOiJodHRwczovL2FwcC5zeXN0ZW1pbml0LmNvbSIsInN1YiI6ImFwaUNsaWVudDo2NDk2Nzc2ZTBjNTE0N2I5YTMwMWNjNjQwMWNhMDgxNiIsImF1ZCI6Imh0dHBzOi8vYXBwLnN5c3RlbWluaXQuY29tIiwiYXBpQ2xpZW50SWQiOiJhcGlDbGllbnQ6NjQ5Njc3NmUwYzUxNDdiOWEzMDFjYzY0MDFjYTA4MTYiLCJiaWxsaW5nQWNjb3VudElkIjoiYmlsbGluZ0FjY291bnQ6ZThmYTNjODI1NzY1NGFkM2E2NzNkZmRiMWU4ZjA3OWUifQ.PBQ9UED1cS3_jU7Q3cTVSz0DPW4rCePBQSXuwv6R0n1dNJ314u34j35OQZLki0-CCPJMTGDHW2CmOZa8hfz9M3PQ6ZoPN0jIpMVAzdLqCHUrUfrztlsyqn8oY8xjS9QWUsVvMoODQYJlQagVdlD90CKWQGYJPgBwDJdX1xfOfhKzHh8UEpHZJh5V-ONzhaEuoMTYJyqYPLTZHv_rV9n7y5g2NB0_1W2Q1pgyB0L-kH7CvrzhLHLdXVkkwBPXqdTRUy1nV9kKNDcoNKUVIlxo6KLqCj8cL0rwFwfHBWiDNdoVwwqjufWf3-6PX3EYHvxE1ciApMRV7Uxs6xNwrl1q7g", DebugFormatter::new());
    let change_run = ChangeRun::new(
        "entity:b1722e33d5824bc392284e2106fea967",
        "system:187d49e584154a9fa64172e8910b3b5b",
        "deploy",
    );
    let command = Command::ChangeRun(change_run);
    client.command(command).await?;
    Ok(())
}
