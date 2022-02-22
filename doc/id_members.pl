#!/usr/bin/perl
use strict;
use warnings;
use autodie;

use Getopt::Long qw();
use YAML::Syck qw();

use Path::Tiny qw();
use Scalar::Util::Numeric qw();
use Bio::Taxon;
use Bio::DB::Taxonomy;

#----------------------------------------------------------#
# GetOpt section
#----------------------------------------------------------#

=head1 NAME

id_members.pl - List members under an ancestor id

=head1 SYNOPSIS

    perl id_members.pl [options] <ancestor id>
      Options:
        --help      -?          brief help message
        --rank          STR     List which rank
        --rankid                Also append rank id
        --td            STR     Path to NCBI taxdmp, default is [~/data/NCBI/taxdmp]

=head1 EXAMPLE

    $ perl taxon/id_members.pl 9606 --rankid
    Homo sapiens neanderthalensis   63221
    Homo sapiens subsp. 'Denisova'  741158

    $ perl taxon/id_members.pl 9605 --rank species
    Homo sapiens
    Homo heidelbergensis

=cut

Getopt::Long::GetOptions(
    'help|?' => sub { Getopt::Long::HelpMessage(0) },
    'rank=s' => \( my $rank ),
    'rankid' => \( my $rankid ),
    'td=s'   => \( my $td_dir = Path::Tiny::path('~/data/NCBI/taxdmp')->stringify ),
) or Getopt::Long::HelpMessage(1);

#----------------------------------------------------------#
# init
#----------------------------------------------------------#
$|++;

# check $rank
my @ranks = qw{species genus family order class phylum kingdom};
my %valid = map { $_ => 1 } @ranks;

if ( $rank and !$valid{$rank} ) {
    die "--rank [$rank} is invalid, it should be one of [@ranks]\n";
}

my $taxon_db;
if ( -e "$td_dir/nodes.dmp" ) {
    $taxon_db = Bio::DB::Taxonomy->new(
        -source    => 'flatfile',
        -directory => $td_dir,
        -nodesfile => "$td_dir/nodes.dmp",
        -namesfile => "$td_dir/names.dmp",
    );
}
else {
    $taxon_db = Bio::DB::Taxonomy->new( -source => 'entrez', );
}

#----------------------------------------------------------#
# start
#----------------------------------------------------------#
for my $ancestor (@ARGV) {
    if ( !Scalar::Util::Numeric::isint($ancestor) ) {
        warn "[$ancestor] doesn't look like a taxonomy id.\n";
        next;
    }

    my $taxon_ancestor = $taxon_db->get_taxon( -taxonid => $ancestor );

    if ( !defined $taxon_ancestor ) {
        warn "Can't find taxon for [$taxon_ancestor]\n";
        next;
    }

    my @taxa = $taxon_db->get_all_Descendents($taxon_ancestor);
    for my Bio::Taxon $taxon (@taxa) {
        if ( defined $rank ) {
            next if $taxon->rank() ne $rank;
        }
        next if $taxon->scientific_name() =~ /environmental/i;
        next if $taxon->scientific_name() =~ /unclassified/i;

        my @row = ();
        push @row, $taxon->scientific_name();
        push @row, $taxon->id() if defined $rankid;
        print join( "\t", @row ), "\n";
    }
}

exit;

__END__
