# Rust Veritech Design

SI is composed of many pieces which, in the end, resolve down to remote function calls.

This document lays out the design of how we will dispatch those remote function calls, 
with a bit of the system architecture as well.

## Veritech Client and Server

The client side of a remote function call lives in `si-model` as a `remote_function`
module. In it are many `RemoteFunctionKinds`, which are bundled together with the `code`
to execute and any other neccessary data (like `inputs` or `args`, etc.). 

The server side is the crate `si-veritech-2` (which will become simply `si-veritech` when
our work is done). 

The two pieces communicate over NATS. The `si-veritech` instances all subscribe to 
`veritech.function.>` as part of the queue group `veritech_dispatch`. The `remote_function`
side sends `RemoteFunctionRequests` to this group, with the `reply` mailbox set. The
`veritech` instance that receives the message will then process it according to the
group it was sent to and the `kind` of remote function execution that was requested
(for example, `resolver` or `qualification`).

Any information that `veritech` wants to send back to `sdf` happens as messages 
on the `reply` mailbox. 

## Execution and Cyclone

In the `RemoteFunctionRequest` payload lives the specific kind of function to execute,
along with a `container_image` and `container_tag` to execute. The container referenced
in this request is then pulled down by `veritech` and run. The specific environment will
evolve over time, and be different for different kinds of requests - but for now, lets
assume it just runs the container on the local host environment via docker.

The container must be configured to start a third program, `si-cyclone`. This program
runs a small warp server with a single responsibility - it knows how to execute function
requests for a given programming language backend, and return the results.

So `veritech` pulls down the `container_image` and starts it, which runs an instance of
`si-cyclone`. When the port is listening on the newly started container, `veritech` starts
a websocket connection to it, and send the `RemoteFunctionRequest` down it.

In the case of, say, `javascript` - `cyclone` will run a small `node.js` or deno program
that has the execution context for our function, and then evals it. I think the best way
to do this would be to wrap the code itself in a javascript module and execute it. The
results of the function then get marshalled back through `cyclone` to the `veritech`
instance that dispatched it via the websocket, which in turn sends the information back
to the `sdf` that dispatched it via its `reply` mailbox. 

How specifically `cyclone` communicates with the interior process is less clear. It could
create its *own* websocket connection to the interior program that executes the function,
and then takes and streams the results back. Or it could be a simpler, line-oriented
protocol - it read the stdout of the interior program, which is emitting JSON payloads,
which then get sent down the socket.

Either way, its this interior program that's responsible for executing the untrusted
user code, running in a container, and streaming the results back.

## Result mapping

Once we can do that dance, we add a `Javascript` ResolverBackend and use the same
code to process the final result as we use for the `JSON` backend. At that point,
we'll be able to write and execute arbitrary functions.

## Short vs long term

Short term, we can just use `docker` to execute the containers. Over time, we're going to
want to use something like `firecracker` to do this work. There's lots of questions about
that - the biggest being it wont' work anymore on macos - but we can work that out when
the time comes. 

For now, if we can get to a place where `veritech` launches containers with `cyclone`
as the program they run, and communicate over websockets to it, then we're money.
Eventually we can change that security profile, or make the backend execution layer
configurable, etc.
