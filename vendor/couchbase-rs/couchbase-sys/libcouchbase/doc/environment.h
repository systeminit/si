/**
 * @page lcb-env-vars-page Environment Variables
 *
 * @brief Environment variables controlling the behavior of the library
 *
 * @details
 * While normally you would configure the library programmatically, there are
 * some environment variables which may affect the library's behavior.
 *
 * Environment variables are intended to aid in debugging and otherwise modifying
 * the library's behavior on a temporary basis. Typically this may be used to
 * increase logging, or to modify/enable a feature which has deliberately been
 * hidden from the normal API.
 *
 *
 * @section lcb-env-vars Environment Variables
 *
 * @subsection LCB_IOPS_NAME
 * Specify the name of the backend to load. This can be one of the built-in
 * names, or it can be a path to a plugin which should be loaded dynamically.
 *
 * The built-in names are:
 *
 * * `libevent`
 * * `libev`
 * * `select`
 * * `libuv`
 * * `iocp` (Windows only)
 *
 * @committed
 *
 *
 * @subsection LCB_IOPS_SYMBOL
 * The symbol inside the shared object (specified as `LCB_IOPS_NAME`) that should
 * contain the plugin initialization function. By default this is in the form of
 * `libcouchbase_create_${NAME}_io_opts`
 *
 * @committed
 *
 * @subsection LCB_DLOPEN_DEBUG
 *
 * Print verbose information to the screen when attempting to log the plugins.
 * This can help determine why a specific plugin is not being loaded
 *
 * @committed
 *
 * @subsection LCB_NO_HTTP
 *
 * Never bootstrap using the HTTP protocol.
 * @volatile
 *
 * @subsection LCB_NO_CCCP
 *
 * Never bootstrap using the memcached protocol
 * @volatile
 *
 * @subsection LCB_LOGLEVEL
 *
 * Enable the console logger and specify its verbosity level. The verbosity
 * level is a number between 1-5 with higher numbers being more verbose
 *
 * @committed
 *
 * @subsection LCB_SSL_MODE
 *
 * Specify the _mode_ to use for SSL. Mode can either be `0` (for no SSL),
 * `1` (for SSL with peer certificate verification), or `3` (for SSL
 * with no certificate verification)
 *
 * @volatile
 *
 * @subsection LCB_SSL_CACERT
 *
 * Specify the path to the CA certificate to be used in order to validate
 * the server's certificate
 *
 * @volatile
 *
 * @section LCB_INTERNAL_ENVVARS Internal Environment Variables
 *
 * @note
 * This section will appear empty unless you have built internal documentation.
 *
 * @internal
 *
 *
 * @subsection LCB_OPTIONS
 *
 * A string containing extra options for the connection string. These options
 * are processed right after the existing connection string options (if any).
 *
 * @internal
 */
