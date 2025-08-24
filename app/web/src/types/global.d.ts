declare module "vue-html-secure";
// some weirdness about the type of timeouts in the browser vs node
// so we create a little helper that is correct no matter where it is used
type Timeout = ReturnType<typeof setTimeout>;

// helpful to have a type alias to tell us things are ISO timestamp strings
type IsoDateString = string;
