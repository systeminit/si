#!/usr/bin/env perl
use strict;
use warnings;
use Digest::MD5 qw(md5_hex);

my $HDR = "** $0 ($$):";
$\="\n";

my $DT_SRC = shift @ARGV;
my $CMD = shift @ARGV;

my @O_FILES = grep { $_ =~ /\.o$/ } @ARGV;
if (!scalar @O_FILES) {
    # Assume this isn't an actual link command?
    print"$HDR Assuming this isn't an LD/AR invocation. Continuing..";
    exec($CMD,@ARGV);
}

# Copy .o files to a temporary location before DTrace messes with them
chomp(my $tmpdir = `mktemp -d -t $$`);
if (system("tar cf - @O_FILES | tar xf - -C $tmpdir") != 0) {
    system("rm -r $tmpdir");
    exit(1);
}

my $ss = join('_', @O_FILES);
my $hexstr = md5_hex($ss);

# From now, we work with files in the temporary location, update @ARGV
map { $_ =~ s,.+\.o$,$tmpdir/$&, } @ARGV;

my $INSTRUMENTED = "generated/probes_${hexstr}_$$.o";
# Run DTrace instrumentation. Assuming running from build directory:
my @args = (
    'dtrace', '-C', '-G',
    '-s', $DT_SRC,
    '-o', $INSTRUMENTED,
    grep { $_ =~ /\.o$/ } @ARGV);

print "$HDR: Creating instrumented DTrace object: @args";
if (system(@args) != 0) {
    system("rm -r $tmpdir");
    exit(1);
}

unshift @ARGV, $CMD;
push @ARGV, $INSTRUMENTED;
print "$HDR: Linking with instrumented DTrace object: @ARGV";
my $rc = system(@ARGV);
system("rm -r $tmpdir");
exit($rc);
