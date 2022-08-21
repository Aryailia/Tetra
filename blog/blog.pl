#!/usr/bin/perl

use v5.28;                     # in perluniintro, latest bug fix for unicode
use feature 'unicode_strings'; # enable perl functions to use Unicode
use utf8;                      # source code in utf8
use strict 'subs';             # only allows declared functions
use warnings;

use Cwd 'abs_path', 'cwd';
use File::Basename 'dirname', 'basename';
use File::Find 'find';
use File::Path 'make_path';
use JSON;
use List::Util 'max';

binmode STDIN, ':encoding(utf8)';
binmode STDOUT, ':encoding(utf8)';
binmode STDERR, ':encoding(utf8)';

chdir dirname($0);
my $CWD = cwd();

# $MAKE reads from "files/" and "templates/"
my $MAKE         = "./make.pl";
my $DEFAULT_LANG = "en";  # Passed to $LANGIFY_EXEC. This sets the directory name of 'ALL'.
my $CACHE_DIR    = ".cache";
my $LANGIFY_DATA = "$CACHE_DIR/langify_metadata.json";

my $LANGIFY_EXEC = "../target/debug/langify";
my $LANG_INP_DIR = "published";
my $LANG_OUT_DIR = "files/blog";

$ENV{'DOMAIN'} = 0;
$ENV{'FORCE'}  = 0;
#run: perl % all

# We are wrapping the commands that build website with `split_langs()`
# This is so `$MAKE build` can occur by itself
my %cmds = (
  "--local" => ["Refer to $MAKE help", sub {
    $ENV{'DOMAIN'} = 1;
    `\Q$MAKE\E --local`;  # Just for the print message
  }],
  "--force" => ["Refer to $MAKE help", sub { $ENV{'FORCE'}  = 1; }],
  "langify" => ["Applies langify on files in $LANG_INP_DIR", sub { split_langs() }],
  "website" => ["Refer to $MAKE help", sub {
    split_langs();
    `\Q$MAKE\E website`;
  }],
  "all" => ["Clean and build everything", sub {
    `\Q$MAKE\E clean build`; my_make("website");
  }],
);

sub split_langs() {
  my $dir = $LANG_INP_DIR;
  make_path($CACHE_DIR);

  my @files;
  find({
    wanted => sub {
      my $path = $File::Find::name;
      return if $path eq $dir;
      # Need basename because `find()` changes directory
      my $base = basename($path);
      return if (!-l $base) && (-d $base);
      push @files, substr($path, length($dir) + 1);
    },

    # Not sure exactly what this needs to be, but enables `perl -T %`
    untaint => sub {},
  }, $dir);

  my %metadata;
  foreach my $p (@files) {
    my $lang_str = `\Q$LANGIFY_EXEC\E \\
      <\Q$dir/$p\E \\
      --default-lang \Q$DEFAULT_LANG\E \\
      \Q$LANG_OUT_DIR\E \Q$p\E`;
    my @lang_list = split /\s/, $lang_str;
    $metadata{$p} = \@lang_list;
  }

  my $json = encode_json \%metadata;
  open FH, '>', "$LANGIFY_DATA";
  print FH $json;
  close FH;
  #say `<\Q$LANGIFY_DATA\E jq '.'`;
}

sub my_make {
  if ($#_  == -1) {
    help();
  } else {
    foreach (@_) {
      if (exists($cmds{$_})) {
        $cmds{$_}[1]();
      } else {
        `\Q$MAKE\E \Q$_\E`;
      }
    }
  }
}

sub help {
  print(<<EOF);
SYNOPSIS
  $0 <subcommand1> [<subcommand2> [..]]

DESCRIPTION
  Functions much like a Makefile

SUBCOMMANDS
EOF
  my $len = max(map { length $_ } keys %cmds);
  for my $key (keys %cmds) {
    printf "  %-${len}s    %s\n", $key, $cmds{$key}[0];
  }
  exit 1;
}

my_make(@ARGV);
