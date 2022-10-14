// some weirdness about the type of timeouts in the browser vs node
// so we create a little helper that is correct no matter where it is used
type Timeout = ReturnType<typeof setTimeout>;
