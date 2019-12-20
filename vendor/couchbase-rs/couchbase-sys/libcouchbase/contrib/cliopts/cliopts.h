#ifndef CLIOPTS_H_
#define CLIOPTS_H_

#include <stddef.h> /* size_t */
#include <limits.h>

#ifdef __cplusplus
extern "C" {
#endif /* __cplusplus */

#if defined(_WIN32) && defined(CLIOPTS_BUILDING_DLL)
#define CLIOPTS_API __declspec( dllexport )

#else
#define CLIOPTS_API
#endif


/**
 * Various option types
 */
typedef enum {
    /** takes no argument, dest should be anything big enough to hold a boolean*/
    CLIOPTS_ARGT_NONE,

    /** simple int type, dest should be an 'int' */
    CLIOPTS_ARGT_INT,

    /** dest should be an unsigned int */
    CLIOPTS_ARGT_UINT,

    /** dest should be an unsigned long long */
    CLIOPTS_ARGT_ULONGLONG,

    /** dest should be an unsigned int, but command line format is hex */
    CLIOPTS_ARGT_HEX,

    /** dest should be a char**. Note that the string is allocated, so you should
     * free() it when done */
    CLIOPTS_ARGT_STRING,

    /** dest should be a float* */
    CLIOPTS_ARGT_FLOAT,

    /**
     * Destination should be cliopts_list. Argument type is assumed to be a
     * string. You can use this option type to build -Doption=value style
     * options which can be processed later on.
     */
    CLIOPTS_ARGT_LIST,

    /**
     * Destination should be cliopts_pair_list. Argument type is assumed to be a
     * string with '=' separator in form of KEY=VALUE.
     */
    CLIOPTS_ARGT_PAIR_LIST
} cliopts_argtype_t;

typedef struct {
    /**
     * Input parameters
     */

    /** Short option, i.e. -v  (0 for none) */
    char kshort;

    /** long option, i.e. --verbose, NULL for none */
    const char *klong;

    /** type of value */
    cliopts_argtype_t ktype;

    /** destination pointer for value */
    void *dest;

    /** help string for this option */
    const char *help;

    /** description of the value, e.g. --file=FILE */
    const char *vdesc;


    /** set this to true if the user must provide this option */
    int required;

    /** set this to true to disable showing the option in the help text */
    int hidden;

    /**
     * Output parameters
     */

    /** whether this option was encountered on the command line */
    int found;

} cliopts_entry;

struct cliopts_extra_settings {
    /** Assume actual arguments start from argv[0], not argv[1] */
    int argv_noskip;
    /** Don't exit on error */
    int error_noexit;
    /** Don't print help on error */
    int error_nohelp;
    /** Don't interpret --help or -? as help flags */
    int help_noflag;
    /** Don't exit on --help or -? */
    int help_noexit;
    /** Program name (defaults to argv[0]) */
    const char *progname;
    /** Usage string (defaults to "[OPTIONS..]") */
    const char *argstring;
    /** Short description (empty by default) */
    const char *shortdesc;
    /** Print default values as well */
    int show_defaults;
    /**
     * Maximum length of a line when printing help. This may be detected
     * using the $COLUMNS environment variable
     */
    int line_max;

    /** Positional parameters (if found). If this array is non-NULL on input
     * then parameters which are not recognized will be placed here. Otherwise
     * the parser will return with an error. This array must be large enough
     * to contain `argc` count strings.
     */
    const char **restargs;

    /** Number of positional parameters (if found) */
    unsigned nrestargs;
};

typedef struct {
    /** Array of string pointers. Allocated via standard malloc functions */
    char **values;
    /** Number of valid entries */
    size_t nvalues;
    /** Number of entries allocated */
    size_t nalloc;
} cliopts_list;

typedef struct {
    /** Array of string pointers. Allocated via standard malloc functions */
    char **keys;
    /** Number of valid entries */
    size_t nkeys;
    /** Array of string pointers. Allocated via standard malloc functions */
    char **values;
    /** Number of valid entries */
    size_t nvalues;
    /** Number of entries allocated */
    size_t nalloc;
} cliopts_pair_list;

/**
 * Clear a list of its contents
 * @param l The list
 */
CLIOPTS_API
void
cliopts_list_clear(cliopts_list *l);

/**
 * Clear a pair list of its contents
 * @param l The pair list
 */
CLIOPTS_API
void
cliopts_pair_list_clear(cliopts_pair_list *l);

/**
 * Parse options.
 *
 * @param entries an array of cliopts_entry structures. The list should be
 * terminated with a structure which has its dest field set to NULL
 *
 * @param argc the count of arguments
 * @param argv the actual list of arguments
 * @param lastidx populated with the amount of elements from argv actually read
 * @params setting a structure defining extra settings for the argument parser.
 * May be NULL
 *
 * @return 0 for success, -1 on error.
 */
CLIOPTS_API
int
cliopts_parse_options(cliopts_entry *entries,
                      int argc,
                      char **argv,
                      int *lastidx,
                      struct cliopts_extra_settings *settings);

/**
 * Split string for arguments, handling single quotes for grouping
 */
CLIOPTS_API
int
cliopts_split_args(char *args, int *argc, char ***argv);

#ifdef __cplusplus
}

#ifdef CLIOPTS_ENABLE_CXX
#include <string>
#include <vector>
#include <list>
#include <cstdlib>
#include <cstring>
#include <cstdio>

namespace cliopts {
class Parser;

/**
 * This class should typically not be used directly. It is a simple wrapper
 * around the C-based ::cliopts_entry class for further wrapping by the
 * cliopts::TOption template class.
 */
class Option : protected cliopts_entry {
public:
    bool passed() const { return found != 0; }
    void setPassed(bool val = true) { found = val ? 1 : 0; }
    int numSpecified() const { return found; }

    Option()
    {
        kshort = 0;
        klong = NULL;
        ktype = CLIOPTS_ARGT_NONE;
        dest = NULL;
        help = NULL;
        vdesc = NULL;
        required = 0;
        hidden = 0;
        found = 0;
    }

private:
    friend class Parser;
};

class EmptyPriv {};

/**
 * Option template class. This class is not meant to be used by applications
 * directly. Applications should use one of the template instantiations
 * below (e.g. cliopts::StringOption)
 *
 * @param T type returned to the application
 * @param Targ integer constant indicating the type of the C argument
 * @param Taccum raw destination type which will store the parsed value
 * @param Tpriv type of private data to be stored for type-specific processing
 */
template <
    typename T,
    cliopts_argtype_t Targ,
    typename Taccum,
    typename Tpriv = EmptyPriv
    >
class TOption : public Option {

private:
    typedef TOption<T,Targ, Taccum, Tpriv> Ttype;
    Taccum innerVal; /**< Pointer for cliopts_entry destination */
    Tpriv priv; /**< Type-specific data */
public:

    /**
     * Construct a new option
     * @param shortname abbreviated short name
     * @param longname long ("GNU-style" name)
     * @param deflval default value to be used
     * @param helpstr Text explaining the option
     */
    TOption(char shortname, const char *longname = NULL,
        T deflval = createDefault(), const char *helpstr = NULL) {

        memset((cliopts_entry *)this, 0, sizeof(cliopts_entry));
        ktype = Targ;
        klong = longname;
        dest = &innerVal;

        abbrev(shortname);
        description(helpstr);
        setDefault(deflval);
    }

    /**
     * Construct a new option
     * @param longname the long ("GNU-Style") name.
     */
    TOption(const char *longname) {
        memset((cliopts_entry *)this, 0, sizeof(cliopts_entry));
        ktype = Targ;
        klong = longname;
        innerVal = createDefault();
        dest = &innerVal;
    }

    /**
     * Copy constructor. This mainly exists to allow chaining (See example)
     * @param other the source option to copy
     */
    TOption(TOption& other) {
        *(cliopts_entry*)this = *(cliopts_entry*) &other;
        innerVal = other.innerVal;
        other.dest = NULL;
        doCopy(other);
        dest = &innerVal;
    }

    ~TOption() {
        freeInnerVal();
    }

    /**
     * Reset result to the default value for the option
     * @return the option object, for method chaining.
     */
    inline Ttype& reset() {
        freeInnerVal();
        innerVal = createDefault();
        priv = Tpriv();
        found = 0;
        return *this;
    }

    /**
     * Set the default value for the option
     * @param val the default value
     * @return the option object, for method chaining.
     */
    inline Ttype& setDefault(const T& val) {
        freeInnerVal();
        innerVal = val;
        return *this;
    }

    /**
     * Set the single-character switch
     * @param val the switch character, e.g. '-v'
     * @return the option object, for method chaining
     */
    inline Ttype& abbrev(char val) { kshort = val; return *this; }

    /**
     * Set the description (or help string) for the option.
     * @param msg The help string e.g. "Increases verbosity"
     * @return the obtion object, for method chaining.
     */
    inline Ttype& description(const char *msg) { help = msg; return *this; }

    /**
     * Set whether this option must appear
     * @param val boolean, set to true if required, false if optional
     * @return the option object, for method chaining
     */
    inline Ttype& mandatory(bool val = true) { required = val; return *this; }

    /**
     * Set the value description string for the option value.
     * @param desc The short description string, e.g. "RETRIES"
     * @return the option object, for method chaining
     */
    inline Ttype& argdesc(const char *desc) { vdesc = desc; return *this; }

    /**
     * Whether to hide this option in the help output
     * @param val true if the option should be hidden
     * @return the object object, for method chaining.
     */
    inline Ttype& hide(bool val = true) { hidden = val; return *this; }

    /**
     * Returns the result object
     * @return a copy of the result object
     */
    inline T result() { return (T)innerVal; }

    /**
     * Returns a reference to the result object
     * @return a reference to the result object.
     */
    inline T& const_result() { return (T)innerVal; }

    operator T() { return result(); }

protected:
    /** Called from within copy constructor */
    inline void doCopy(TOption&) {}

    inline void freeInnerVal() {}

    /** Create the default value for the option */
    static inline Taccum createDefault() { return Taccum(); }
};

typedef TOption<std::string,
        CLIOPTS_ARGT_STRING,
        const char*,
        std::string> StringOption;

typedef TOption<std::vector<std::string>,
        CLIOPTS_ARGT_LIST,
        cliopts_list,
        std::vector<std::string> > ListOption;

typedef TOption<std::vector<std::pair<std::string, std::string> >,
        CLIOPTS_ARGT_PAIR_LIST,
        cliopts_pair_list,
        std::vector<std::pair<std::string, std::string> > > PairListOption;

typedef TOption<bool,
        CLIOPTS_ARGT_NONE,
        int> BoolOption;

typedef TOption<unsigned,
        CLIOPTS_ARGT_UINT,
        unsigned> UIntOption;

typedef TOption<unsigned long long,
        CLIOPTS_ARGT_ULONGLONG,
        unsigned long long> ULongLongOption;

typedef TOption<int,
        CLIOPTS_ARGT_INT,
        int> IntOption;

typedef TOption<int,
        CLIOPTS_ARGT_HEX,
        unsigned> HexOption;

typedef TOption<float,
        CLIOPTS_ARGT_FLOAT,
        float> FloatOption;

// STRING ROUTINES
template<> inline std::string& StringOption::const_result() {
    if (innerVal && passed()) {
        priv = innerVal;
    }
    return priv;
}
template<> inline std::string StringOption::result() {
    return const_result();
}
template<> inline void StringOption::freeInnerVal() {
    free((void *)innerVal);
    innerVal = NULL;
}
template<> inline StringOption& StringOption::setDefault(const std::string& s) {
    priv = s;
    freeInnerVal();
    innerVal = strdup(priv.c_str());
    return *this;
}
template<> inline void StringOption::doCopy(StringOption& other) {
    priv = other.priv;
    if (other.innerVal == other.priv.c_str()) {
        freeInnerVal();
        innerVal = strdup(priv.c_str());
    }
}
template<> inline const char* StringOption::createDefault() { return NULL; }

// LIST ROUTINES
template<> inline std::vector<std::string>& ListOption::const_result() {
    if (priv.empty()) {
        for (size_t ii = 0; ii < innerVal.nvalues; ii++) {
            priv.push_back(innerVal.values[ii]);
        }
    }
    return priv;
}
template<> inline std::vector<std::string> ListOption::result() {
    return const_result();
}
template<> inline void ListOption::freeInnerVal() { cliopts_list_clear(&innerVal); }

struct PairListDtor {
    static void call(void *arg) {
        cliopts_pair_list_clear((cliopts_pair_list *)arg);
    }
};

// PAIR LIST ROUTINES
template<> inline std::vector<std::pair<std::string, std::string> >& PairListOption::const_result() {
    if (priv.empty()) {
        for (size_t ii = 0; ii < innerVal.nvalues; ii++) {
            priv.push_back(std::make_pair(innerVal.keys[ii], innerVal.values[ii]));
        }
    }
    return priv;
}
template<> inline std::vector<std::pair<std::string, std::string> > PairListOption::result() {
    return const_result();
}
template<> inline void PairListOption::freeInnerVal() { cliopts_pair_list_clear(&innerVal); }

// BOOL ROUTINES
template<> inline BoolOption& BoolOption::setDefault(const bool& b) {
    innerVal = b ? 1 : 0; return *this;
}
template<> inline bool BoolOption::result() {
    return innerVal != 0 ? true : false;
}

/**
 * Parser class which contains one or more cliopts::Option objects. Options
 * should be added via the #addOption() member function.
 */
class Parser {
public:
    /**
     * Construct a new parser
     * @param name the "program name" which is printed at the top of the
     * help message.
     */
    Parser(const char *name = NULL) {
        memset(&default_settings, 0, sizeof default_settings);
        default_settings.progname = name;
    }

    /**
     * Adds an option to the parser. The option is then checked for presence
     * on the commandline (in #parse()).
     * @param opt the option to add. Note that the application is responsible
     * for keeping the option in valid memory.
     */
    void addOption(Option *opt) { options.push_back(opt); }

    void addOption(Option& opt) { options.push_back(&opt); }

    /**
     * Resets internal state.
     */
    void reset() {
        options.clear();
        restargs.clear();
    }

    /**
     * Parses the options from the commandline
     * @param argc number of arguments
     * @param argv list of arguments
     * @param standalone_args whether to accept (and store) positional arguments
     * (after all named options are processed).
     * @return true on parse success, false on parse failure
     */
    bool parse(int argc, char **argv, bool standalone_args = false) {
        std::vector<cliopts_entry> ents;
        cliopts_extra_settings settings = default_settings;
        int lastix;

        for (unsigned ii = 0; ii < options.size(); ++ii) {
            ents.push_back(*options[ii]);
        }

        if (ents.empty()) { return false; }
        ents.push_back(Option());
        const char **tmpargs = NULL;
        if (standalone_args) {
            tmpargs = new const char*[argc];
            settings.restargs = tmpargs;
            settings.nrestargs = 0;
        }
        settings.show_defaults = 1;

        int rv = cliopts_parse_options(&ents[0], argc, argv, &lastix, &settings);

        if (tmpargs != NULL) {
            for (unsigned ii = 0; ii < settings.nrestargs; ii++) {
                restargs.push_back(tmpargs[ii]);
            }
            delete[] tmpargs;
        }

        // Copy the options back
        for (unsigned ii = 0; ii < options.size(); ii++) {
            *(cliopts_entry *)options[ii] = ents[ii];
        }

        if (rv == 0 && lastix != 0) {
            for (; lastix < argc; lastix++) {
                restargs.push_back(argv[lastix]);
            }
        }

        return rv == 0;
    }

    /**
     * Get the list of any positional arguments found on the commandline
     * @return A list of positional arguments found.
     */
    const std::vector<std::string>& getRestArgs() { return restargs; }

    cliopts_extra_settings default_settings;
private:
    std::vector<Option*> options;
    std::vector<std::string> restargs;
    Parser(Parser&);
};
} // namespace
#endif /* CLIOPTS_ENABLE_CXX */

#endif /* __cplusplus */

#endif /* CLIOPTS_H_ */
