// # Kinds
//   - Conditionals
//     - serialized execution of steps
//     - stops workflow if some step throws, propagates throw to the caller
//     - concatenates hardcoded input args of next step with return value from previous step (even if next step is another workflow)
//       - non-root workflows pass their input argument concatenated to the first step
//     - return value from the last step is propagated to the caller
//   - Exception
//     - serialized execution of steps
//     - stops workflow if a step doesn't throw (succeeds)
//     - if every step throws, the last throw is propagated to the caller
//     - concatenates hardcoded input args of next step with thrown error from previous step (even if next step is another workflow)
//     - propagates returned value from last step ran
//   - Parallel
//     - parallel execution of steps
//     - waits for all of the steps to execute, if any step throws propagates the first throw to the caller
//       - non-root workflows pass their argument concatenated to all steps
//     - return values are ignored

const commands = require("./commands");
const workflows = require("./workflow");

exports.process = async (workflow, lastReturn) => {
  const handles = [];

  for (const step of workflow.steps) {
    if (step.workflow) {
      if (workflow.kind === "parallel") {
        handles.push(exports.workerize("process")(await workflows[step.workflow](...(step.args ?? []), lastReturn), lastReturn));
      } else if (workflow.kind === "conditional") {
        lastReturn = await exports.process(await workflows[step.workflow](...(step.args ?? []), lastReturn), lastReturn);
      } else if (workflow.kind === "exception") {
        try {
          return await exports.process(await workflows[step.workflow](...(step.args ?? []), lastReturn), lastReturn);
        } catch (err) {
          lastReturn = err;
        }
      }
    } else {
      if (workflow.kind === "parallel") {
        handles.push(exports.workerize(commands[step.command])(...(step.args ?? []), lastReturn));
      } else if (workflow.kind === "conditional"){
        lastReturn = await commands[step.command](...(step.args ?? []), lastReturn);
      } else if (workflow.kind === "exception") {
        try {
          return await commands[step.command](...(step.args ?? []), lastReturn);
        } catch (err) {
          lastReturn = err;
        }
      }
    }
  }

  // If all failed, propagates last throw to the caller
  if (workflow.kind === "exception") {
    throw lastReturn;
  } 

  // Waits for all to execute, even if some fail
  // Propagates first throw to the caller
  for (const result of await Promise.allSettled(handles)) {
    // Only reached if kind is parallel
    if (result.status === "rejected") throw result.reason;
  }
}

exports.workerize = (fn, workerOptions) => {
  const { Worker } = require('worker_threads')
  return function(...workerData) {
    return new Promise((resolve, reject) => {
      const worker = new Worker(`
        const { workerize } = require('./executor');
        exports.workerize = workerize;
        const { workerData, parentPort } = require('worker_threads')
        Promise.resolve((${fn.toString()})(...workerData)).then(async returnedData => {
          parentPort.postMessage(returnedData)
        })
      `, { ...workerOptions, eval: true, workerData })

      worker.on('message', resolve)
      worker.on('error', reject)
      worker.on('exit', code => {
        if (code === 0) {
          resolve(null)
        } else {
          reject(new Error(`Worker stopped with exit code ${code}`))
        }
      })
    })
  }
}
