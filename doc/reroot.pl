#!/usr/bin/perl
use strict;
use warnings;
use autodie;

use Bio::TreeIO;

my $file  = shift;
my $label = shift;

my $in = Bio::TreeIO->new(
    -format => 'newick',
    -file   => $file
);
my $out = Bio::TreeIO->new( -format => 'newick' );

while ( my $t = $in->next_tree ) {
    my ($n) = $t->find_node( -id => $label );
    $t->reroot($n);
    $out->write_tree($t);
}
