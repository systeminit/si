/**
 * @page intro_sec Introduction
 *
 * libcouchbase is an asynchronous library for connecting to a Couchbase
 * server and performing data operations.
 *
 * This contains the API documentation for the library. The documentation
 * consists of both _internal_ and _public_ interfaces.
 *
 * Using the library is comprised of these steps:
 *
 * 1. Create an instance (See lcb_create())
 * 2. Install callbacks (See lcb_set_get_callback())
 * 3. Schedule an operation (e.g. lcb_get())
 * 4. Wait for the operation to complete (lcb_wait())
 *
 * _libcouchbase_ is an asynchronous library which means that operation
 * results are passed to callbacks you define rather than being returned from
 * functions.
 *
 * Callbacks are passed a `cookie` parameter which is a user-defined pointer
 * (i.e. your own pointer which can be `NULL`) to associate a specific command
 * with a specific callback invocation.
 *
 * For simple synchronous use, you will need to call lcb_wait() after each
 * set of scheduled operations. During lcb_wait() the library will block for
 * I/O and invoke your callbacks as the results for the operations arrive.
 *
 * For non-synchronous use cases you can integrate with a variety of event loops
 * via the various plugins, or integrate one yourself via the `IOPS` API
 * (see @ref lcb-io-plugin-api)
 *
 * Modifying the library's settings (for example, timeout settings) may be done
 * via the lcb_cntl() interface (see @ref lcb-cntl-settings) or via some environment
 * variables (see @ref lcb-env-vars)
 *
 *
 *
 * ## Using the headers and libraries
 *
 * Using the libcouchbase headers is simple. Simply:
 * @code{.c}
 * #include <libcouchbase/couchbase.h>
 * @endcode
 *
 * into your application.
 *
 * To link, simply link against `libcouchbase` (e.g. `-lcouchbase`).
 *
 *
 * See @ref lcb_attributes for interface stability taxonomy and @ref lcb_thrsafe
 * for information on programming in threaded environments
 *
 *
 *
 * ## Minimal example usage
 * @include example/minimal/minimal.c
 *
 * ## Configuring and Tuning the library
 *
 * The library may be configured either programmatically via lcb_cntl(),
 * or via the environment (see @ref lcb-env-vars)
 *
 *
 * ## Simple Usage Steps
 *
 * 1. Create an `lcb_t` handle. This is done via lcb_create()
 * 2. Schedule the initial connection, this is done via lcb_connect()
 * 3. Wait for the initial connection to complete, via lcb_wait()
 * 4. Install the callbacks for retrieval and storage (lcb_set_get_callback(),
 *    lcb_set_stor_callback()).
 * 5. Set up a command structure for storing an item, i.e. @ref lcb_store_cmd_t.
 * 6. Schedule the operation via lcb_store(). You will also likely want to
 *    pass a `cookie` parameter along with it so that you can associate your
 *    application's structures via the callback.
 * 7. Invoke lcb_wait(). Your callback will be invoked with the result of
 *    the storage operation.
 *
 *
 *
 *
 * @internal
 * ## Public vs Internal APIs
 *
 * The @ref lcb-public-api section is where you should begin browsing to develop
 * with the library. Any sections not contained within the public API are
 * internal and are provided to aid in developing new features and fixing bugs
 * within the library itself.
 *
 * ## Internal Header Layouts
 *
 * The internal headers are organized like so:
 *
 * * <lcbio/lcbio.h> - I/O Core
 * * <mc/mcreq.h> - Memcached packet codecs
 * * <netbuf/netbuf.h> - Write buffer implementation
 * * <rdb/rope.h> - Read buffer implementation
 * * <mcserver/mcserver.h> - Memcached client I/O
 * * <mcserver/negotiate.h> - Memcached client initial SASL handling
 * * <bucketconfig/clconfig.h> - Couchbase cluster configuration retrieval
 * * <packetutils.h> - Utility for response packets
 * * <src/retryq.h> - Retry queue for failed packets
 * * <src/internal.h> - Other internal functions not in the above categories
 *
 * In addition to these files, there are several non-core files which exist
 * to provide simple utilities which are not specific to the library:
 *
 * * <list.h> - Double-linked list
 * * <sllist.h>, <sllist-inl.h> - Single linked list
 * * <genhash.h> - Hashtable
 * * <hashset.h> - Set of unique elements
 * * <hostlist.h> - Host/Port structures and lists
 *
 * @endinternal
 *
 * ## Prerequisite Knowledge
 *
 * libcouchbase is a cross platform C library used to interact with Couchbase
 * Server. It is assumed that you know about:
 *
 * * Key-Value stores
 * * The C language
 * * Asynchronous and non-blocking programming.
 *
 * To develop with the I/O integration APIs, you will need to know about:
 *
 * * Socket APIs
 * * Event loops
 *
 */
