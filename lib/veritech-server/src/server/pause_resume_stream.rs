// TODO(nick,fletcher): make this generic and in its own crate.

use std::{process::Command, task::Poll, time::Duration};

use futures::{future::BoxFuture, FutureExt, StreamExt};
use pin_project_lite::pin_project;
use si_data_nats::async_nats::jetstream::{
    consumer::{self, Consumer, StreamError},
    Message,
};
use telemetry::prelude::*;
use telemetry_utils::metric;
use tokio::sync::mpsc;

#[remain::sorted]
#[derive(Debug, Clone, Copy)]
enum PauseResumeCommand {
    // NOTE(nick,fletcer): consider adding "PauseWithDuration".
    Pause,
    Resume,
}

/// A controller for pausing and resuming a [`PauseResumeStream`].
#[derive(Debug, Clone)]
pub struct PauseResumeController {
    inner: mpsc::UnboundedSender<PauseResumeCommand>,
}

impl PauseResumeController {
    /// Pauses a [`PauseResumeStream`] with a stream-side timer.
    pub fn pause(&self) {
        // TODO(nick,fletcher): disambiguate the difference between "I failed to send due to an
        // unexpected failure" vs. "I failed to send because we are shutting down and the rx has
        // already closed for valid reasons".
        if let Err(err) = self.inner.send(PauseResumeCommand::Pause) {
            error!(si.error.message = ?err, "error when sending pause command");
        }
    }

    /// Resumes a [`PauseResumeStream`].
    #[allow(dead_code)]
    pub fn resume(&self) {
        // TODO(nick,fletcher): disambiguate the difference between "I failed to send due to an
        // unexpected failure" vs. "I failed to send because we are shutting down and the rx has
        // already closed for valid reasons".
        if let Err(err) = self.inner.send(PauseResumeCommand::Resume) {
            error!(si.error.message = ?err, "error when sending resume command");
        }
    }
}

#[remain::sorted]
enum State {
    ReadyToSubscribe,
    Subscribed(consumer::pull::Stream),
    Subscribing(BoxFuture<'static, Result<consumer::pull::Stream, StreamError>>),
    Unsubscribed(BoxFuture<'static, ()>),
}

pin_project! {
    /// A stream that wraps and inner stream with the ability to "pause" the inner stream.
    pub struct PauseResumeStream {
        consumer: Consumer<consumer::pull::Config>,
        incoming: State,
        rx: mpsc::UnboundedReceiver<PauseResumeCommand>,
        tx: mpsc::UnboundedSender<PauseResumeCommand>,
        last_unprocessed_command: Option<PauseResumeCommand>,
        pause_duration: Duration,
        reconnect_backoff_duration: Duration,
    }
}

impl PauseResumeStream {
    pub fn new(
        consumer: Consumer<consumer::pull::Config>,
        pause_duration: Duration,
        reconnect_backoff_duration: Duration,
    ) -> Self {
        // NOTE(nick,fletcher): evaluate making this bounded.
        let (tx, rx) = mpsc::unbounded_channel();
        Self {
            consumer,
            incoming: State::ReadyToSubscribe,
            rx,
            tx,
            last_unprocessed_command: None,
            pause_duration,
            reconnect_backoff_duration,
        }
    }

    pub fn controller(&self) -> PauseResumeController {
        PauseResumeController {
            inner: self.tx.clone(),
        }
    }
}

impl futures::Stream for PauseResumeStream {
    type Item = Result<Message, consumer::pull::MessagesError>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let project = self.project();

        // TODO(nick,brit): restore or delete pause/resume.
        // // Drain everything in the controller channel until we reach the last unprocessed command.
        // // We don't have to worry about the order of the messages because if handlers need to issue
        // // a new command in the future, then they will.
        // loop {
        //     match project.rx.poll_recv(cx) {
        //         // We have received a "pause" command and we should keep going until the last command
        //         Poll::Ready(Some(PauseResumeCommand::Pause)) => {
        //             trace!(previous_last_unprocessed_command = ?project.last_unprocessed_command, "changing last command to pause");
        //             *project.last_unprocessed_command = Some(PauseResumeCommand::Pause);
        //             continue;
        //         }
        //         // We have received a "resume" command and we should keep going until the last command
        //         Poll::Ready(Some(PauseResumeCommand::Resume)) => {
        //             trace!(previous_last_unprocessed_command = ?project.last_unprocessed_command, "changing last command to resume");
        //             *project.last_unprocessed_command = Some(PauseResumeCommand::Resume);
        //             continue;
        //         }
        //         // The channel stream has closed
        //         Poll::Ready(None) => {
        //             trace!("the channel stream has closed for monitoring commands within the pause resume stream");
        //             break;
        //         }
        //         // No more messages inbounded, but the channel stream is alive
        //         // This is the critical path!
        //         Poll::Pending => break,
        //     }
        // }
        // trace!(last_unprocessed_command = ?project.last_unprocessed_command, "determined last unprocessed command");

        // Now that we've stored the last unprocessed command, we can determine what to do with the
        // inner stream.
        loop {
            match project.incoming {
                // If we need the initial state, we state transition to "subscribing"
                State::ReadyToSubscribe => {
                    let subscribing = {
                        // We need an owned consumer for the "messages" call. We also need to move that
                        // consumer into the pin. If this hurts to look at, you brain is working.
                        let cloned_consumer_needed_for_messages_call = project.consumer.clone();
                        Box::pin(async move {
                            cloned_consumer_needed_for_messages_call.messages().await
                        })
                    };

                    // Set to subscribing and continue the loop
                    debug!("ready to subscribe --> subscribing");
                    *project.incoming = State::Subscribing(subscribing);
                }
                // We are subscribed!
                // This is the critical path!
                State::Subscribed(stream) => match stream.poll_next_unpin(cx) {
                    Poll::Ready(msg) => match msg {
                        // If we are missing a heartbeat, we need to unsubscribe.
                        // TODO(nick): decide is this is too invasive into the "pause-resume" pattern.
                        Some(Err(err))
                            if err.kind()
                                == consumer::pull::MessagesErrorKind::MissingHeartbeat =>
                        {
                            metric!(counter.veritech.pause_resume_stream.missing_heartbeat = 1);
                            error!("fatal: encountered missing heartbeat");
                            perform_verinuke();
                        }
                        // For all other errors and successful cases, return ready!
                        // This the critical path!
                        _ => return Poll::Ready(msg),
                    },
                    Poll::Pending => return Poll::Pending,
                    // TODO(nick,brit): restore or delete pause/resume.
                    //     match project.last_unprocessed_command {
                    //         // If we received a pause command and are subscribed, we need to unsubscribe.
                    //         // We do this by replacing the mutably borrowed state wrapper and (hopefully)
                    //         // "dropping" the old stream. We need to reset the last unprocessed command
                    //         // since we've "used" it.
                    //         Some(PauseResumeCommand::Pause) => {
                    //             *project.last_unprocessed_command = None;
                    //             debug!("subscribed --> pause --> unsubscribed");
                    //             *project.incoming = State::Unsubscribed(Box::pin(tokio::time::sleep(
                    //                 project.pause_duration.to_owned(),
                    //             )));
                    //         }
                    //         // If we received resume command, we are good to keep on truckin'! We need to
                    //         // reset the last unprocessed command since we've "used" it.
                    //         Some(PauseResumeCommand::Resume) => {
                    //             *project.last_unprocessed_command = None;
                    //             match stream.poll_next_unpin(cx) {
                    //                 Poll::Ready(Some(msg)) => match msg {
                    //                     // If we are missing a heartbeat, we need to unsubscribe.
                    //                     // TODO(nick): decide is this is too invasive into the "pause-resume" pattern.
                    //                     Err(err) if err.kind() == consumer::pull::MessagesErrorKind::MissingHeartbeat => {
                    //                         metric!(counter.veritech.pause_resume_stream.missing_heartbeat = 1);
                    //                         error!("fatal: encountered missing heartbeat");
                    //                         perform_verinuke();
                    //                     },
                    //                     // For all other errors and successful cases, return ready!
                    //                     _ => return Poll::Ready(Some(msg)),
                    //                 },
                    //                 Poll::Ready(None) => return Poll::Ready(None),
                    //                 Poll::Pending => return Poll::Pending,
                    //             }
                    //         }
                    //         // If we received a no command command, we are good to keep on truckin'!
                    //         // This is the critical path!
                    //         None => match stream.poll_next_unpin(cx) {
                    //             Poll::Ready(Some(msg)) => match msg {
                    //                 // If we are missing a heartbeat, we need to unsubscribe.
                    //                 // TODO(nick): decide is this is too invasive into the "pause-resume" pattern.
                    //                 Err(err)
                    //                     if err.kind()
                    //                         == consumer::pull::MessagesErrorKind::MissingHeartbeat =>
                    //                 {
                    //                     metric!(
                    //                         counter.veritech.pause_resume_stream.missing_heartbeat = 1
                    //                     );
                    //                     error!("fatal: encountered missing heartbeat");
                    //                     perform_verinuke();
                    //                 }
                    //                 // For all other errors and successful cases, return ready!
                    //                 // This the critical path!
                    //                 _ => return Poll::Ready(Some(msg)),
                    //             },
                    //             Poll::Ready(None) => return Poll::Ready(None),
                    //             Poll::Pending => return Poll::Pending,
                    //         },
                    //     }
                    // }
                },
                // Let's try to transition from "subscribing" to "subscribed"
                State::Subscribing(subscribing) => match subscribing.poll_unpin(cx) {
                    // Success! The stream is ready. We can return poll pending.
                    Poll::Ready(Ok(stream)) => {
                        debug!("subscribing --> ready --> subscribed ");
                        *project.incoming = State::Subscribed(stream);
                    }
                    // Epic fail! We can't connect to the stream. Let's backoff and try again.
                    Poll::Ready(Err(stream_error)) => {
                        error!(si.error.message = ?stream_error, reconnect_backoff_duration = ?project.reconnect_backoff_duration, "hit stream error while trying to subscribe within pause resume stream (backing off and trying again)");
                        debug!("subscribing --> ready with stream error --> unsubscribed");
                        *project.incoming = State::Unsubscribed(Box::pin(tokio::time::sleep(
                            project.reconnect_backoff_duration.to_owned(),
                        )));
                    }
                    // Still waiting for the stream to be ready...
                    Poll::Pending => return Poll::Pending,
                },
                // We are unsubscribed and we need to see if we've finished with our sleep
                State::Unsubscribed(sleep) => match sleep.poll_unpin(cx) {
                    // If we are ready to go, let's loop again and start subscribing
                    Poll::Ready(()) => {
                        debug!("unsubscribed --> sleep done --> ready to subscribe");
                        *project.incoming = State::ReadyToSubscribe
                    }
                    // Still sleeping!
                    Poll::Pending => return Poll::Pending,
                },
            }
            debug!("looping within pause resume stream");
        }
    }
}

// The gods have abandoned us...
//
// DO NOT copy this pattern anywhere else. We should be using something like tokio signal for this
// kind of thing (e.g. sdf), but we don't want to pollute the veritech server code at the time of
// writing since we believe verinuke will be removed.
fn perform_verinuke() {
    let pid = std::process::id();
    error!(%pid, "fatal: verinuke requested... running 'kill -15' (SIGTERM) on our own process (warning: veritech exit code may be zero rather than non-zero)");

    // NOTE(nick): the "si-service" crate sees SIGTERM and will return "Ok(())", which will make
    // veritech exit with exit code 0. However, we believe we will always exceed the shutdown
    // timeout and we'll have a non-zero exit code anyway.
    let _result = Command::new("kill").arg("-15").arg(pid.to_string()).spawn();
}
