/* -*- Mode: C; tab-width: 4; c-basic-offset: 4; indent-tabs-mode: nil -*- */

#ifndef _WIN32
#include <sys/ioctl.h>
#include <termios.h>
#else
#include <windows.h>
#endif

#include <string.h>
#include <stdlib.h>
#include <stdio.h>
#include <limits.h>
#include <errno.h>
#include <ctype.h>

#include "cliopts.h"


enum {
    CLIOPTS_ERR_SUCCESS,
    CLIOPTS_ERR_NEED_ARG,
    CLIOPTS_ERR_ISSWITCH,
    CLIOPTS_ERR_BADOPT,
    CLIOPTS_ERR_BAD_VALUE,
    CLIOPTS_ERR_UNRECOGNIZED
};

struct cliopts_priv {
    cliopts_entry *entries;

    cliopts_entry *prev;
    cliopts_entry *current;
    struct cliopts_extra_settings *settings;

    char *errstr;
    int errnum;

    int argsplit;
    int wanted;

#define MAX_KEYLEN 4096
    char current_key[MAX_KEYLEN];
    char current_value[MAX_KEYLEN];
};

enum {
    WANT_OPTION,
    WANT_VALUE,

    MODE_ERROR,
    MODE_RESTARGS,
    MODE_HELP
};

#define INDENT "  "

#ifdef CLIOPTS_DEBUG

#define cliopt_debug(...) \
    fprintf(stderr, "(%s:%d) ", __func__, __LINE__); \
    fprintf(stderr, __VA_ARGS__); \
    fprintf(stderr, "\n")

#else
/** variadic macros not c89 */
static void cliopt_debug(void *unused, ...) { (void)unused; }
#endif /* CLIOPT_DEBUG */

static int
parse_option(struct cliopts_priv *ctx, const char *key);


static int
parse_value(struct cliopts_priv *ctx, const char *value);

static void ensure_list_capacity(cliopts_list *l)
{
    if (l->nvalues == l->nalloc) {
        if (l->nalloc == 0) {
            l->nalloc = 2;
            l->values = malloc(l->nalloc * sizeof(*l->values));
        } else {
            l->nalloc *= 1.5;
            l->values = realloc(l->values, sizeof(*l->values) * l->nalloc);
        }
    }
}

static void
add_list_value(const char *src, size_t nsrc, cliopts_list *l)
{
    char *cp = malloc(nsrc + 1);

    ensure_list_capacity(l);
    l->values[l->nvalues++] = cp;
    cp[nsrc] = '\0';
    memcpy(cp, src, nsrc);
}

CLIOPTS_API
void
cliopts_list_clear(cliopts_list *l)
{
    size_t ii;
    for (ii = 0; ii < l->nvalues; ii++) {
        free(l->values[ii]);
    }
    free(l->values);
    l->values = NULL;
    l->nvalues = 0;
    l->nalloc = 0;
}

static void ensure_pair_list_capacity(cliopts_pair_list *l)
{
    if (l->nvalues == l->nalloc) {
        if (l->nalloc == 0) {
            l->nalloc = 2;
            l->keys = malloc(l->nalloc * sizeof(*l->keys));
            l->values = malloc(l->nalloc * sizeof(*l->values));
        } else {
            l->nalloc *= 1.5;
            l->keys = realloc(l->keys, sizeof(*l->keys) * l->nalloc);
            l->values = realloc(l->values, sizeof(*l->values) * l->nalloc);
        }
    }
}

static void
add_pair_list_value(const char *src, size_t nsrc, cliopts_pair_list *l)
{
    char *key = NULL, *val = NULL;
    char *sep = memchr(src, '=', nsrc);
    if (sep == NULL) {
        key = malloc(nsrc + 1);
        memcpy(key, src, nsrc);
        key[nsrc] = '\0';
        val = malloc(1);
        val[0] = '\0';
    } else {
        char *pp = sep;
        size_t nkey = sep - src;
        size_t nval = nsrc - nkey - 1;
        for (; pp > src; pp--, nkey--) {
            if (*pp != ' ' && *pp != '\t' && *pp != '\0') {
                break;
            }
        }
        key = malloc(nkey + 1);
        memcpy(key, src, nkey);
        key[nkey] = '\0';
        val = malloc(nval + 1);
        memcpy(val, sep + 1, nval);
        val[nval] = '\0';
    }

    ensure_pair_list_capacity(l);

    l->keys[l->nvalues] = key;
    l->values[l->nvalues] = val;
    l->nvalues++;
}

CLIOPTS_API
void
cliopts_pair_list_clear(cliopts_pair_list *l)
{
    size_t ii;
    for (ii = 0; ii < l->nvalues; ii++) {
        free(l->keys[ii]);
        free(l->values[ii]);
    }
    free(l->keys);
    free(l->values);
    l->keys = NULL;
    l->values = NULL;
    l->nvalues = 0;
    l->nalloc = 0;
}

/**
 * Various extraction/conversion functions for numerics
 */

#define _VERIFY_INT_COMMON(m1, m2) \
    if (value == m1 || value > m2) { *errp = "Value too large"; return -1; } \
    if (*endptr != '\0') { *errp = "Trailing garbage"; return -1; }

static int
extract_int(const char *s, void *dest, char **errp)
{
    long int value;
    char *endptr = NULL;
    value = strtol(s, &endptr, 10);
    _VERIFY_INT_COMMON(LONG_MAX, INT_MAX)
    *(int*)dest = value;
    return 0;
}

static int
extract_uint(const char *s, void *dest, char **errp)
{
    unsigned long int value;
    char *endptr = NULL;
    value = strtoul(s, &endptr, 10);
    _VERIFY_INT_COMMON(ULONG_MAX, UINT_MAX)
    *(unsigned int*)dest = value;
    return 0;
}

#ifdef ULLONG_MAX
static int
extract_ulonglong(const char *s, void *dest, char **errp)
{
    unsigned long long value;
    char *endptr = NULL;
#ifdef _WIN32
    value = _strtoui64(s, &endptr, 10);
#else
    value = strtoull(s, &endptr, 10);
#endif
    _VERIFY_INT_COMMON(ULLONG_MAX, ULLONG_MAX);
    *(unsigned long long *)dest = value;
    return 0;
}
#else
static int extract_ulonglong(const char *s, void *dest, char **errp)
{
    *errp = "long long not available";
    return -1;
}
#endif /* ULLONG_MAX */

static int
extract_hex(const char *s, void *dest, char **errp)
{
    unsigned long value;
    char *endptr = NULL;
    value = strtoul(s, &endptr, 16);
    _VERIFY_INT_COMMON(ULONG_MAX, UINT_MAX);
    *(unsigned int*)dest = value;
    return 0;
}

#undef _VERIFY_INT_COMMON

static int
extract_float(const char *s, void *dest, char **errp)
{
    char dummy_buf[4096];
    float value;
    if (sscanf(s, "%f%s", &value, dummy_buf) != 1) {
        *errp = "Found trailing garbage";
        return -1;
    }
    *(float*)dest = value;
    return 0;
}

typedef int(*cliopts_extractor_func)(const char*, void*, char**);


/**
 * This function tries to extract a single value for an option key.
 * If it successfully has extracted a value, it returns MODE_VALUE.
 * If the entry takes no arguments, then the current string is a key,
 * and it will return MODE_OPTION. On error, MODE_ERROR is set, and errp
 * will point to a string.
 *
 * @param entry The current entry
 * @param value the string which might be a value
 * @errp a pointer which will be populated with the address of the error, if any
 *
 * @return a MODE_* type
 */
static int
parse_value(struct cliopts_priv *ctx,
            const char *value)
{
    cliopts_entry *entry = ctx->current;

    size_t vlen = strlen(value);
    cliopts_extractor_func exfn = NULL;
    int exret;
    int is_option = 0;

    cliopt_debug("Called with %s, want=%d", value, ctx->wanted);

    if (ctx->argsplit) {
        if (vlen > 2 && strncmp(value, "--", 2) == 0) {
            is_option = 1;
        } else if (*value == '-') {
            is_option = 1;
        }
    }

    if (is_option) {
        ctx->errstr = "Expected option. Got '-' or '--' prefixed value "
                        "(use = if this is really a value)";
        ctx->errnum = CLIOPTS_ERR_NEED_ARG;
        return MODE_ERROR;
    }

    if (entry->ktype == CLIOPTS_ARGT_STRING) {
        char *vp = malloc(vlen+1);
        vp[vlen] = 0;
        strcpy(vp, value);
        free(*(char**)entry->dest);
        *(char**)entry->dest = vp;
        return WANT_OPTION;
    }

    if (entry->ktype == CLIOPTS_ARGT_LIST) {
        add_list_value(value, vlen, (cliopts_list *)entry->dest);
        return WANT_OPTION;
    }
    if (entry->ktype == CLIOPTS_ARGT_PAIR_LIST) {
        add_pair_list_value(value, vlen, (cliopts_pair_list *)entry->dest);
        return WANT_OPTION;
    }

    if (entry->ktype == CLIOPTS_ARGT_FLOAT) {
        exfn = extract_float;
    } else if (entry->ktype == CLIOPTS_ARGT_HEX) {
        exfn = extract_hex;
    } else if (entry->ktype == CLIOPTS_ARGT_INT) {
        exfn = extract_int;
    } else if (entry->ktype == CLIOPTS_ARGT_UINT) {
        exfn = extract_uint;
    } else if (entry->ktype == CLIOPTS_ARGT_ULONGLONG) {
        exfn = extract_ulonglong;
    } else {
        fprintf(stderr, "Unrecognized type %d.\n", entry->ktype);
        return MODE_ERROR;
    }

    exret = exfn(value, entry->dest, &ctx->errstr);
    if (exret == 0) {
        return WANT_OPTION;
    } else {
        ctx->errnum = CLIOPTS_ERR_BAD_VALUE;
    }

    return MODE_ERROR;
}

/**
 * Like parse_value, except for keys.
 *
 * @param entries all option entries
 * @param key the current string from argv
 * @param errp a pointer which will be populated with the address of an error
 * string
 *
 * @param found_entry a pointer to be populated with the relevant entry
 * structure
 * @param kp a pointer which will be poplated with the address of the 'sanitized'
 * key string
 *
 * @param valp if the string is actually a key-value pair (i.e. --foo=bar) then
 * this will be populated with the address of that string
 *
 * @return MODE_OPTION if an option was found, MODE_VALUE if the current option
 * is a value, or MODE_ERROR on error
 */
static int
parse_option(struct cliopts_priv *ctx,
          const char *key)
{
    cliopts_entry *cur = NULL;
    int prefix_len = 0;
    unsigned ii = 0;
    const char *valp = NULL;
    size_t klen;

    klen = strlen(key);
    ctx->errstr = NULL;
    ctx->prev = ctx->current;
    ctx->current = NULL;

    cliopt_debug("Called with %s, want=%d", key, ctx->wanted);
    if (klen == 0) {
        ctx->errstr = "Got an empty string";
        ctx->errnum = CLIOPTS_ERR_BADOPT;
        return MODE_ERROR;
    }
    if (klen > MAX_KEYLEN) {
        ctx->errstr = "The key is to big";
        ctx->errnum = CLIOPTS_ERR_BADOPT;
        return MODE_ERROR;
    }

    /**
     * figure out what type of option it is..
     * it can either be a -c, --long, or --long=value
     */
    while (*key == '-') {
        key++;
        prefix_len++;
        klen--;
    }

    for (ii = 0; ii < klen; ii++) {
        if (key[ii] == '"' || key[ii] == '\'') {
            ii = klen;
            break;

        } else if (key[ii] == '=' && prefix_len == 2) {
            /* only split on '=' if we're called as '--' */
            valp = key + (ii + 1);
            klen = ii;
            break;
        }
    }
    if (valp && strlen(valp) > MAX_KEYLEN) {
        ctx->errstr = "The value is to big";
        ctx->errnum = CLIOPTS_ERR_BAD_VALUE;
        return MODE_ERROR;
    }

    GT_PARSEOPT:
    memset(ctx->current_value, 0, sizeof(ctx->current_value));
    memcpy(ctx->current_key, key, klen);
    ctx->current_key[ii] = '\0';

    if (valp) {
        strcpy(ctx->current_value, valp);
    }

    if (prefix_len == 0 || prefix_len > 2) {
        if (ctx->settings->restargs) {
            key -= prefix_len;
            ctx->settings->restargs[ctx->settings->nrestargs++] = key;
            return WANT_OPTION;
        } else if (ctx->prev && ctx->prev->ktype == CLIOPTS_ARGT_NONE) {
            ctx->errstr = "Option does not accept a value";
            ctx->errnum = CLIOPTS_ERR_ISSWITCH;
        } else {
            ctx->errstr = "Options must begin with either '-' or '--'";
            ctx->errnum = CLIOPTS_ERR_BADOPT;
        }
        return MODE_ERROR;
    }

    /**
     * --help or -?
     */

    if ( (prefix_len == 1 && *key == '?') ||
            (prefix_len == 2 && strcmp(key, "help") == 0)) {
        return MODE_HELP;
    }

    /**
     * Bare --
     */
    if (prefix_len == 2 && *key == '\0') {
        if (ctx->settings->restargs) {

        }
        if (ctx->wanted == WANT_VALUE) {
            ctx->errnum = CLIOPTS_ERR_NEED_ARG;
            ctx->errstr = "Found bare '--', but value wanted";
            return MODE_ERROR;
        }

        return MODE_RESTARGS;
    }

    for (cur = ctx->entries; cur->dest; cur++) {
        int optlen;
        if (prefix_len == 1) {
            if (cur->kshort == ctx->current_key[0]) {
                ctx->current = cur;
                break;
            }
            continue;
        }
        /** else, prefix_len is 2 */
        if (cur->klong == NULL ||
                (optlen = strlen(cur->klong) != klen) ||
                strncmp(cur->klong, ctx->current_key, klen) != 0) {

            continue;
        }

        ctx->current = cur;
        break;
    }

    if (!ctx->current) {
        ctx->errstr = "Unknown option";
        ctx->errnum = CLIOPTS_ERR_UNRECOGNIZED;
        return MODE_ERROR;
    }

    ctx->current->found++;
    if (ctx->current->ktype != CLIOPTS_ARGT_NONE) {
        ctx->wanted = WANT_VALUE;
    }

    if (ctx->current_value[0]) {
        /* --foo=bar */
        if (ctx->current->ktype == CLIOPTS_ARGT_NONE) {
            ctx->errnum = CLIOPTS_ERR_ISSWITCH;
            ctx->errstr = "Option takes no arguments";
            return MODE_ERROR;
        } else {
            return parse_value(ctx, ctx->current_value);
        }
    }

    if (ctx->current->ktype == CLIOPTS_ARGT_NONE) {
        *(char*)ctx->current->dest = 1;

        if (prefix_len == 1 && klen > 1) {
            /**
             * e.g. ls -lsh
             */
            klen--;
            key++;

            /**
             * While we can also possibly recurse, this may be a security risk
             * as it wouldn't take much to cause a deep recursion on the stack
             * which will cause all sorts of nasties.
             */
            goto GT_PARSEOPT;
        }
        return WANT_OPTION;

    } else if (prefix_len == 1 && klen > 1) {

        /* e.g. patch -p0 */
        ctx->wanted = WANT_VALUE;
        return parse_value(ctx, key + 1);
    }
    return WANT_VALUE;
}

static char *
get_option_name(cliopts_entry *entry, char *buf)
{
    /* [-s,--option] */
    char *bufp = buf;
    bufp += sprintf(buf, "[");
    if (entry->kshort) {
        bufp += sprintf(bufp, "-%c", entry->kshort);
    }
    if (entry->klong) {
        if (entry->kshort) {
            bufp += sprintf(bufp, ",");
        }
        bufp += sprintf(bufp, "--%s", entry->klong);
    }
    sprintf(bufp, "]");
    return buf;
}

static int get_terminal_width(void)
{
#ifndef _WIN32
    struct winsize max;
    if (ioctl(0, TIOCGWINSZ, &max) != -1) {
        return max.ws_col;
    } else {
        return 80;
    }
#else
    CONSOLE_SCREEN_BUFFER_INFO cbsi;
    GetConsoleScreenBufferInfo(GetStdHandle(STD_OUTPUT_HANDLE), &cbsi);
    return cbsi.srWindow.Right - cbsi.srWindow.Left;
#endif
}

static char*
format_option_help(cliopts_entry *entry,
                   char *buf,
                   struct cliopts_extra_settings *settings)
{
    char *bufp = buf;
    if (entry->kshort) {
        bufp += sprintf(bufp, " -%c ", entry->kshort);
    }

#define _advance_margin(offset) \
    while(bufp-buf < offset || *bufp) { \
        if (!*bufp) { \
            *bufp = ' '; \
        } \
        bufp++; \
    }

    _advance_margin(4)

    if (entry->klong) {
        bufp += sprintf(bufp, " --%s ", entry->klong);
    }

    if (entry->vdesc) {
        bufp += sprintf(bufp, " <%s> ", entry->vdesc);
    }

    _advance_margin(35)
#undef _advance_margin

    if (entry->help) {
        unsigned initial_indent = bufp - buf + 1;
        int curpos = initial_indent;
        const char *help_p = entry->help;

        for (; *help_p; help_p++, curpos++, bufp++) {

            if (curpos >= settings->line_max) {
                unsigned ii;
                if (!isspace(*help_p) && !isspace(*(help_p-1))) {
                    *bufp = '-';
                    bufp++;
                }
                *bufp = '\n';
                bufp++;

                for (ii = 0; ii < initial_indent+1; ii++, bufp++) {
                    *bufp = ' ';
                }

                curpos = initial_indent;
                if (isspace(*help_p)) {
                    bufp--;
                    continue;
                }
            }
            *bufp = *help_p;
        }
    }

    *bufp = '\0';
    return buf;
}

static void
print_help(struct cliopts_priv *ctx, struct cliopts_extra_settings *settings)
{
    cliopts_entry *cur;
    cliopts_entry helpent = { 0 };
    char helpbuf[1024] = { 0 };

    helpent.klong = "help";
    helpent.kshort = '?';
    helpent.help = "this message";

    fprintf(stderr, "Usage:\n");
    fprintf(stderr, "  %s %s\n\n", settings->progname, settings->argstring);
    if (settings->shortdesc) {
        fprintf(stderr, "%s", settings->shortdesc);
        fprintf(stderr, "\n");
    }


    for (cur = ctx->entries; cur->dest; cur++) {
        if (cur->hidden) {
            continue;
        }

        memset(helpbuf, 0, sizeof(helpbuf));
        format_option_help(cur, helpbuf, settings);
        fprintf(stderr, INDENT "%s", helpbuf);


        if (settings->show_defaults) {
            fprintf(stderr, " [Default=");

            switch (cur->ktype) {
            case CLIOPTS_ARGT_STRING:
                fprintf(stderr, "'%s'", (cur->dest && *(char **)cur->dest) ?
                        *(char**)cur->dest : "");
                break;
            case CLIOPTS_ARGT_LIST: {
                size_t ii;
                cliopts_list *l = (cliopts_list *)cur->dest;
                for (ii = 0; ii < l->nvalues; ii++) {
                    fprintf(stderr, "'%s'", l->values[ii]);
                    if (ii != l->nvalues-1) {
                        fprintf(stderr, ", ");
                    }
                }
                break;
            }
            case CLIOPTS_ARGT_PAIR_LIST: {
                size_t ii;
                cliopts_pair_list *l = (cliopts_pair_list *)cur->dest;
                for (ii = 0; ii < l->nvalues; ii++) {
                    fprintf(stderr, "'%s=%s'", l->keys[ii], l->values[ii]);
                    if (ii != l->nvalues-1) {
                        fprintf(stderr, ", ");
                    }
                }
                break;
            }
            case CLIOPTS_ARGT_FLOAT:
                fprintf(stderr, "%0.2f", *(float*)cur->dest);
                break;
            case CLIOPTS_ARGT_HEX:
                fprintf(stderr, "0x%x", *(int*)cur->dest);
                break;
            case CLIOPTS_ARGT_INT:
                fprintf(stderr, "%d", *(int*)cur->dest);
                break;
            case CLIOPTS_ARGT_UINT:
                fprintf(stderr, "%u", *(unsigned int*)cur->dest);
                break;
#ifdef ULLONG_MAX
            case CLIOPTS_ARGT_ULONGLONG:
                fprintf(stderr, "%llu", *(unsigned long long*)cur->dest);
                break;
#endif
            case CLIOPTS_ARGT_NONE:
                fprintf(stderr, "%s", *(int*)cur->dest ? "TRUE" : "FALSE");
                break;
            default:
                fprintf(stderr, "Unknown option type '%d'", (int)cur->ktype);
                break;
            }
            fprintf(stderr, "]");
        }
        fprintf(stderr, "\n");
    }
    memset(helpbuf, 0, sizeof(helpbuf));
    fprintf(stderr, INDENT "%s\n",
            format_option_help(&helpent, helpbuf, settings));

}

static void
dump_error(struct cliopts_priv *ctx)
{
    fprintf(stderr, "Couldn't parse options: %s\n", ctx->errstr);
    if (ctx->errnum == CLIOPTS_ERR_BADOPT) {
        fprintf(stderr, "Bad option: %s", ctx->current_key);
    } else if (ctx->errnum == CLIOPTS_ERR_BAD_VALUE) {
        fprintf(stderr, "Bad value '%s' for %s",
                ctx->current_value,
                ctx->current_key);
    } else if (ctx->errnum == CLIOPTS_ERR_UNRECOGNIZED) {
        fprintf(stderr, "No such option: %s", ctx->current_key);
    } else if (ctx->errnum == CLIOPTS_ERR_ISSWITCH) {
        char optbuf[64] = { 0 };
        fprintf(stderr, "Option %s takes no arguments",
                get_option_name(ctx->prev, optbuf));
    }
    fprintf(stderr, "\n");

}

CLIOPTS_API
int
cliopts_parse_options(cliopts_entry *entries,
                      int argc,
                      char **argv,
                      int *lastidx,
                      struct cliopts_extra_settings *settings)
{
    /**
     * Now let's build ourselves a
     */
    int curmode;
    int ii, ret = 0, lastidx_s = 0;
    struct cliopts_priv ctx = { 0 };
    struct cliopts_extra_settings default_settings = { 0 };

    if (!lastidx) {
        lastidx = &lastidx_s;
    }

    ctx.entries = entries;

    if (!settings) {
        settings = &default_settings;
        settings->show_defaults = 1;
    }
    if (!settings->progname) {
        settings->progname = argv[0];
    }
    if (!settings->argstring) {
        settings->argstring = "[OPTIONS...]";
    }
    settings->nrestargs = 0;

    if (!settings->line_max) {
        settings->line_max = get_terminal_width() - 3;
    }

    ii = (settings->argv_noskip) ? 0 : 1;

    if (ii >= argc) {
        *lastidx = 0;
        ret = 0;
        goto GT_CHECK_REQ;
        return 0;
    }

    curmode = WANT_OPTION;
    ctx.wanted = curmode;
    ctx.settings = settings;

    for (; ii < argc; ii++) {

        if (curmode == WANT_OPTION) {
            curmode = parse_option(&ctx, argv[ii]);
        } else if (curmode == WANT_VALUE) {
            curmode = parse_value(&ctx, argv[ii]);
        }

        if (curmode == MODE_ERROR) {
            if (settings->error_nohelp == 0) {
                dump_error(&ctx);
            }
            ret = -1;
            break;
        } else if (curmode == MODE_HELP) {
            if (settings->help_noflag) {
                /* ignore it ? */
                continue;
            }

            print_help(&ctx, settings);
            if (!settings->help_noexit) {
                exit(0);
            }

        } else if (curmode == MODE_RESTARGS) {
            ii++;
            break;
        } else {
            ctx.wanted = curmode;
        }
    }

    *lastidx = ii;

    if (curmode == WANT_VALUE) {
        ret = -1;

        if (settings->error_nohelp == 0) {
            fprintf(stderr,
                    "Option %s requires argument\n",
                    ctx.current_key);
        }
        goto GT_RET;
    }

    GT_CHECK_REQ:
    {
        cliopts_entry *cur_ent;
        for (cur_ent = entries; cur_ent->dest; cur_ent++) {
            char entbuf[128] = { 0 };
            if (cur_ent->found || cur_ent->required == 0) {
                continue;
            }

            ret = -1;
            if (settings->error_nohelp) {
                goto GT_RET;
            }

            fprintf(stderr, "Required option %s missing\n",
                    get_option_name(cur_ent, entbuf));
        }
    }

    GT_RET:
    if (ret == -1) {
        if (settings->error_nohelp == 0) {
            print_help(&ctx, settings);
        }
        if (settings->error_noexit == 0) {
            exit(EXIT_FAILURE);
        }
    }
    return ret;
}

CLIOPTS_API
int cliopts_split_args(char *args, int *argc, char ***argv)
{
    char *p = args;
    char *current = NULL;
    int skip = 0;

    *argc = 0;
    *argv = NULL;

#define ch(x) (*(x + skip))

    while (1) {

        while (ch(p) && isspace(ch(p))) {
            p++;
        }
        if (*p) {
            int insq = 0;
            int done = 0;

            if (current == NULL) {
                current = p;
            }
            while (!done) {
                if (insq) {
                    if (ch(p) == '\\' && ch(p + 1) == '\'') {
                        skip++;
                    } else if (ch(p) == '\'') {
                        if (ch(p + 1) && !isspace(ch(p + 1))) {
                            // do not consider single quote inside word as terminating
                        } else {
                            *p = '\0';
                            p++;
                            done = 1;
                        }
                    } else if (!*p) {
                        cliopt_debug("unterminated single quote");
                        goto err;
                    }
                } else {
                    switch (ch(p)) {
                    case '\'':
                        if (current == p) {
                            current++;
                            insq = 1;
                        }
                        break;
                    case ' ':
                    case '\n':
                    case '\r':
                    case '\t':
                        *p = '\0';
                        p++;
                    case '\0':
                        done = 1;
                        break;
                    }
                }
                if (skip) {
                    *p = ch(p);
                }
                if (*p && !done) {
                    p++;
                }
            }
            *argv = realloc(*argv, (*argc + 1) * sizeof(char *));
            (*argv)[*argc] = current;
            (*argc)++;
            current = NULL;
        } else {
            return 0;
        }
    }

err:
    free(*argv);
    *argc = 0;
    *argv = NULL;
    return 1;
}
