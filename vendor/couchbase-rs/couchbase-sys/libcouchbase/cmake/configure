#!/usr/bin/env perl

# This script will eventually become the top-level configure directory ONCE
# we've completely migrated away from autotools

use strict;
use warnings;
use Getopt::Long;
use File::Path;
use File::Basename qw(dirname);
use Cwd qw(getcwd realpath);
use FindBin qw($Bin $RealBin);

# Format here is:
# [ Argument Name,
#   Option format string,
#   Option description,
#   Default format string,
#   Default value (ref)
# ]

my @HELP_DESCRIPTIONS = (
    [ "enable-debug", "",
        "Create a debug build",
        "%d", \(my $DEBUG = 0) ],

    [ "enable-profiler", "",
        "Build with profiler support (gperftools)",
        "%d", \(my $USE_PROFILER = 0) ],

    [ "with-ninja", "s",
        "Generate Ninja Makefiles",
        "%s", \(my $USE_NINJA = "") ],

    [ "prefix", "s",
        "installation prefix",
        "%s", \(my $PREFIX = "/usr/local") ],

    [ "disable-tools", "",
        "Don't build tools",
        "%d", \(my $NO_TOOLS = 0)],

    [ "disable-tests", "",
        "Don't build tests",
        "%d", \(my $NO_TESTS = 0)],

    [ "with-libuv", "s",
        "Path to LIBUV installation",
        "%s", \(my $LIBUV_ROOT = "")],

    [ "with-libev", "s",
        "Path to LIBEV installation",
        "%s", \(my $LIBEV_ROOT = "")],

    [ "with-libevent", "s",
        "Path to LIBEVENT installation",
        "%s", \(my $LIBEVENT_ROOT = "")],

    [ "with-cmake", "s",
        "Executable to use as cmake",
        "%s", \(my $CMAKE_EXE = "cmake")],

    [ "with-openssl", "s",
        "Path to OpenSSL installation",
        "%s", \(my $OPENSSL_ROOT = "")],

    [ "disable-plugins", "",
        "Disable building of external plugins",
        "%d", \(my $NO_PLUGINS = 0)],

    [ "disable-cxx", "",
        "Disable C++ targets",
        "%d", \(my $NO_CXX = 0) ],
    [ "disable-couchbasemock", "",
        "Don't run tests with Mock",
        "%d", \(my $NO_MOCK = 0) ],
    [ "enable-maintainer-mode", "",
        "Maintainer mode. Everything is strict",
        "%d", \(my $MAINTAINER_MODE = 0)],

    [ "enable-fat-binary", "",
        "Build universal binaries on OS X",
        "%d", \(my $OSX_UNIVERSAL_BUILD = 0)],

    [ "cmake-verbose", "",
        "Verbose CMake output",
        "%d", \(my $CMAKE_VERBOSE = 0)],

    [ "host", "s",
        "Set host target for Win32 cross compilation",
        "%s", \(my $CMAKE_HOST = "")],

    [ "enable-static", "",
        "Build static libraries",
        "%d", \(my $STATIC_LIBS = 0)],
    [ "enable-embedded-libevent-plugin", "",
        "Embed the libevent plugin into the library's core",
        "%d", \(my $EMBED_LIBEVENT_PLUGIN = 0) ],
    [ "cc", "s",
        "C Compiler",
        "%s", \(my $CC = "") ],

    [ "cxx", "s",
        "C++ Compiler",
        "%s", \(my $CXX = "") ],
    [ "disable-ssl", "",
        "Do not build SSL support",
        "%d", \(my $NO_SSL = 0)],
    [ "enable-asan", "",
        "Enable LLVM AddressSanitizer support",
        "%d", \(my $LCB_USE_ASAN = 0)],
    [ "enable-coverage", "",
        "Compile with code coverage instrumentation",
        "%d", \(my $LCB_USE_COVERAGE = 0)],
    [ "enable-examples", "",
        "Build examples",
        "%s", \(my $BUILD_EXAMPLES = 0)],
    [ "build-dir", "s",
        "Directory to build in",
        "%s", \(my $BUILD_DIR = "build") ],
    [ "skip-git-version", "",
        "Skip version detection using git",
        "%d", \(my $SKIP_GIT_VERSION = 0) ],

    [ "help", "",
        "This message", "%s", \(my $WANT_HELP = 0) ]
);

my @HELPLINES = ();
my %GETOPT_ARGS = ();

foreach my $optdesc (@HELP_DESCRIPTIONS) {
    my ($name, $vfmt, $desc, $deflfmt, $vref) = @$optdesc;
    my $go_key = $name;
    if ($vfmt) {
        $go_key .= "=$vfmt";
    }
    $GETOPT_ARGS{$go_key} = $vref;
}

GetOptions(%GETOPT_ARGS);

if ($WANT_HELP) {
    printf("configure.pl ... options\n");
    foreach my $optdesc (@HELP_DESCRIPTIONS) {
        my ($name, $vfmt, $desc, $fmt, $deflref) = @$optdesc;
        printf("  --%-20s", $name);
        if (length($name) > 18) {
            printf("\n");
            printf("%-20s%s\n","", $desc);
        } else {
            printf("%s.", $desc);
        }
        printf(" Default=".$fmt."\n", ${$deflref});
    }

    exit(0);
}

my $origdir = getcwd();
my $srcdir = realpath("$RealBin/..");

if ($origdir eq $srcdir) {
    printf("Detected in-source build. Making '$BUILD_DIR' directory\n");
    mkpath($BUILD_DIR);
    chdir($BUILD_DIR);
}

if ($NO_CXX) {
    $NO_TESTS=1;
    $NO_TOOLS=1;
}

my @CM_OPTIONS = ();

my @VARNAMES = (qw(CC CXX CFLAGS CPPFLAGS CXXFLAGS LDFLAGS LIBS));
my %ENVOPTIONS = ();
foreach my $var (@VARNAMES) {
    $ENVOPTIONS{$var} = $ENV{$var};
}

foreach my $arg (@ARGV) {
    my ($k,$v) = $arg =~ m/([^=]+)=(.*)/;
    unless ($k && $v && exists $ENVOPTIONS{$k}) {
        die("Invalid variable $arg");
    }
    $ENVOPTIONS{$k} = $v;
}

my $flags_common = "";
my $flags_cxx = "";
my $flags_cc = "";

if ($ENVOPTIONS{CC}) { $CC = $ENVOPTIONS{CC}; }
if ($ENVOPTIONS{CXX}) { $CXX = $ENVOPTIONS{CXX}; }
if ($ENVOPTIONS{CPPFLAGS}) {
    $flags_common = $ENVOPTIONS{CPPFLAGS};
}
$flags_cxx = $flags_common;
$flags_cc = $flags_common;
if ($ENVOPTIONS{CFLAGS}) {
    $flags_cc .= " " . $ENVOPTIONS{CFLAGS};
}
if ($ENVOPTIONS{CXXFLAGS}) {
    $flags_cxx .= " ".$ENVOPTIONS{CXXFLAGS};
}
if ($flags_cc) {
    push @CM_OPTIONS, "-DCMAKE_C_FLAGS=$flags_cc";
}
if ($flags_cxx) {
    push @CM_OPTIONS, "-DCMAKE_CXX_FLAGS=$flags_cxx";
}
if ($ENVOPTIONS{LDFLAGS}) {
    # Flags for the linker
    my $flags = $ENVOPTIONS{LDFLAGS};
    foreach my $var (qw(EXE_LINKER_FLAGS SHARED_LINKER_FLAGS)) {
        push @CM_OPTIONS, "-DCMAKE_${var}=$flags";
    }
}
if ($ENVOPTIONS{LIBS}) {
    warn("Don't know how to convert autotools 'LIBS' into CMake");
}

if ($DEBUG) {
    push @CM_OPTIONS, "-DCMAKE_BUILD_TYPE=DEBUG";
} else {
    push @CM_OPTIONS, "-DCMAKE_BUILD_TYPE=RelWithDebInfo"
}

if ($USE_PROFILER) {
    push @CM_OPTIONS, "-DLCB_USE_PROFILER=1";
}

if ($PREFIX) {
    push @CM_OPTIONS, "-DCMAKE_INSTALL_PREFIX=$PREFIX";
}
if ($NO_TESTS) {
    push @CM_OPTIONS, "-DLCB_NO_TESTS=1";
}
if ($NO_TOOLS) {
    push @CM_OPTIONS, "-DLCB_NO_TOOLS=1";
}

if ($SKIP_GIT_VERSION) {
    push @CM_OPTIONS, "-DLCB_SKIP_GIT_VERSION=1"
}

if ($NO_PLUGINS) {
    push @CM_OPTIONS, "-DLCB_NO_PLUGINS=1";
}
if ($LIBUV_ROOT) {
    push @CM_OPTIONS, "-DLIBUV_ROOT=$LIBUV_ROOT";
}
if ($LIBEV_ROOT) {
    push @CM_OPTIONS, "-DLIBEV_ROOT=$LIBEV_ROOT";
}
if ($MAINTAINER_MODE) {
    push @CM_OPTIONS, "-DLCB_MAINTAINER_MODE=1";
}
if ($OSX_UNIVERSAL_BUILD) {
    push @CM_OPTIONS, "-DLCB_UNIVERSAL_BINARY=1";
}
if ($CMAKE_VERBOSE) {
    push @CM_OPTIONS, "--debug-output";
}
if ($STATIC_LIBS) {
    push @CM_OPTIONS, "-DBUILD_SHARED_LIBS=OFF", "-DLCB_BUILD_STATIC=1";
}
if ($CC) {
    push @CM_OPTIONS, "-DCMAKE_C_COMPILER=$CC";
}
if ($CXX) {
    push @CM_OPTIONS, "-DCMAKE_CXX_COMPILER=$CXX";
}
if ($USE_NINJA) {
    push @CM_OPTIONS, "-DCMAKE_MAKE_PROGRAM=$USE_NINJA";
    push @CM_OPTIONS, "-GNinja";
} else {
    push @CM_OPTIONS, "-GUnix Makefiles";
}
if ($NO_SSL) {
    push @CM_OPTIONS, "-DLCB_NO_SSL=$NO_SSL";
}
if ($OPENSSL_ROOT) {
    push @CM_OPTIONS, "-DOPENSSL_ROOT_DIR=$OPENSSL_ROOT";
}
if ($LCB_USE_ASAN) {
    push @CM_OPTIONS, "-DLCB_USE_ASAN=1";
}
if ($LCB_USE_COVERAGE) {
    push @CM_OPTIONS, "-DLCB_USE_COVERAGE=1";
}
if ($NO_MOCK) {
    push @CM_OPTIONS, "-DLCB_NO_MOCK=1";
}
if ($EMBED_LIBEVENT_PLUGIN) {
    push @CM_OPTIONS, "-DLCB_EMBED_PLUGIN_LIBEVENT=1";
}
if ($BUILD_EXAMPLES) {
    push @CM_OPTIONS, "-DLCB_BUILD_EXAMPLES=1";
}

if ($CMAKE_HOST) {
    # Write the toolchain file
    my $fname = "TOOLCHAIN-$CMAKE_HOST.cmake";
    open my $fp, ">", $fname;
    print $fp <<EOF;
SET(CMAKE_SYSTEM_NAME Windows)
SET(CMAKE_C_COMPILER $CMAKE_HOST-gcc)
SET(CMAKE_CXX_COMPILER $CMAKE_HOST-g++)
SET(CMAKE_RC_COMPILER $CMAKE_HOST-windres)
SET(CMAKE_FIND_ROOT_PATH /usr/$CMAKE_HOST)
SET(CMAKE_FIND_ROOT_PATH_MODE_PROGRAM NEVER)
SET(CMAKE_FIND_ROOT_PATH_MODE_LIBRARY ONLY)
SET(CMAKE_FIND_ROOT_PATH_MODE_INCLUDE ONLY)
EOF
    $fname = realpath($fname);
    push @CM_OPTIONS, "-DCMAKE_TOOLCHAIN_FILE=$fname";
}

print "$CMAKE_EXE ../ \\ \n";
foreach my $opt (@CM_OPTIONS) {
    printf("    \"%s\" \\ \n", $opt);
}

unshift @CM_OPTIONS, "$CMAKE_EXE", $srcdir;
my $rv = system(@CM_OPTIONS);
if ($rv != 0) {
    die("Couldn't invoke cmake");
}

if ($origdir eq $srcdir) {
    chdir($origdir);
    open(my $fh, ">", "Makefile");
    print $fh <<EOF;
all:

\%::
\t\$(MAKE) -C $BUILD_DIR \$\@

include $BUILD_DIR/defs.mk
include packaging/rpm/package.mk
include packaging/deb/package.mk

EOF
    close($fh);
}
