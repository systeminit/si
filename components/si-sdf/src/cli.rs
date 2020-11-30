// The way this will work:
//
// The command line interface will:
//  * Create a new CLI::Client
//     -> Create a websocket, which is resovled in Cli::Server
//  * Send one Command over the websocket
//  * Stream the response back
//  * Close the socket
//
//  cliFormatter();
//  let client = cli::Client::new("wss:///");
//  let rx = client.command(Command::Deploy(...)).await?;
//  while let Some(message) = rx.next().await {
//      cliFormatter.from_event(event);
//      cliFormatter.from_event_log(event);
//      cliFormatter.from_output_line(evetn);
//  ...
//  }
//
//  How it works: client -> server on websocket.
//  client sends Command, server tracks event that starts the chain
//  server forwards all events, eventLogs and outputlines that relate to the event
//  server closes the connection when the event is ended
//

pub mod client;
pub mod formatter;
pub mod server;
