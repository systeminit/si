# Dependent Values Update Debouncer

## High Level Algorithm

The debouncer task coordinates with other debouncer tasks via a NATS KV store
which acts as a distributed lock (or could be though of as a leader election).
Each key corresponds to a unique change set within a workspace. That is, the
key is of the form `{workspace_pk}.{change_set_id}`. Additionally, there is a
`max_age` set for all keys which will naturally age out if no
update/delete/purge operations are performed. This "aging out" of a key should
only happen when a debouncer task crashes or panics without properly cleaning
up its state *or* when a Rebaser service is disconnected from its network or
connection to NATS (i.e. a network partition).

The general algorithm for each debouncer task is as follows:

```mermaid
stateDiagram-v2
    Wait: Waits to become leader
    WatchKey: Watch key for delete/age out
    AttemptsLeader: Attempt to become leader
    Leader: Becomes leader (owns key)
    LeaderWaits: Wait for debounce window
    CheckIfPending: Check if values are pending
    PendingValues: Prepares to run DVU
    SetRunningStatus: Set key value status to "running"
    PerformDvu: Enqueue, run, and block on DVU
    KeepaliveWaits: Wait for keepalive window
    UpdateKey: Update key preventing age out

    [*] --> Wait
    Wait --> WatchKey
    WatchKey --> AttemptsLeader: Key deleted
    WatchKey --> AttemptsLeader: Key aged out
    AttemptsLeader --> WatchKey: Loses key acquire
    AttemptsLeader --> Leader: Wins key acquire
    Leader --> Wait: Returns to waiting
    state Leader {
        [*] --> LeaderWaits
        LeaderWaits --> CheckIfPending: Tick time elapses
        CheckIfPending --> LeaderWaits: No pending values
        CheckIfPending --> PendingValues: Pending values found
        PendingValues --> SetRunningStatus
        SetRunningStatus --> PerformDvu
        PerformDvu --> DeleteKey
        DeleteKey --> [*]
        --
        [*] --> KeepaliveWaits
        KeepaliveWaits --> UpdateKey: Tick time elapsed
        UpdateKey --> KeepaliveWaits
    }
```
