#!/usr/bin/perl
use strict;
use warnings;
use autodie;

use Getopt::Long;
use Config::Tiny;
use FindBin;
use YAML::Syck;

use Path::Tiny;
use Scalar::Util::Numeric qw(isint);
use Bio::DB::Taxonomy;

#----------------------------------------------------------#
# GetOpt section
#----------------------------------------------------------#
my $Config = Config::Tiny->read("$FindBin::RealBin/../config.ini");

=head1 NAME

id_project_to.pl - Project taxonomy ids to names, species, genus or higher ranks.

=head1 SYNOPSIS

    cat <file> | perl id_project_to.pl [options]
      Options:
        --help      -?          brief help message
        --column    -c  INT     column order where ids are, start from 1
        --separator -s  STR     separator of the line, default is "\s+"
        --rank          STR     Project to which rank, default is scientific name.
        --rankid                Also append rank id

=head1 EXAMPLE

    $ echo 9606 | perl taxon/id_project_to.pl
    9606,Homo sapiens

    $ echo 9606 | perl taxon/id_project_to.pl --rank class
    9606,Mammalia

    $ echo 9606 Human | perl taxon/id_project_to.pl --rank class --rankid
    9606,Human,Mammalia,40674

    $ echo Human,9606  | perl taxon/id_project_to.pl -c 2 -s ","
    Human,9606,Homo sapiens

=cut

# running options
my $td_dir = path( $Config->{path}{td} )->stringify;    # taxdmp

GetOptions(
    'help|?' => sub { Getopt::Long::HelpMessage(0) },
    'column|c=i'    => \( my $column    = 1 ),
    'separator|s=s' => \( my $separator = '\s+' ),
    'rank=s'        => \my $rank,
    'rankid'        => \my $rankid,
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
while ( my $line = <> ) {
    chomp $line;
    my @row = split /$separator/, $line;

    my $taxon_id = $row[ $column - 1 ];

    if ( isint($taxon_id) ) {
        my $strain = $taxon_db->get_taxon( -taxonid => $taxon_id );
        if ( !$strain ) {
            warn "Can't find taxon for [$taxon_id]\n";
            push @row, 'NA';
            push @row, '0' if $rankid;
        }
        else {
            if ($rank) {
                my $taxon_obj = find_ancestor( $strain, $rank );
                if ( !$taxon_obj ) {
                    warn "Can't find [$rank] for [$taxon_id]\n";
                    push @row, 'NA';
                    push @row, '0' if $rankid;
                }
                else {
                    push @row, $taxon_obj->scientific_name;
                    push @row, $taxon_obj->id if $rankid;
                }
            }
            else {
                push @row, $strain->scientific_name;
                push @row, $strain->id if $rankid;
            }
        }
    }
    else {
        warn "[$taxon_id] doesn't look like a taxonmy id.\n";
        push @row, 'NA';
        push @row, '0' if $rankid;
    }

    print join( ",", @row ), "\n";
}

exit;

#----------------------------------------------------------#
# Subroutines
#----------------------------------------------------------#
sub find_ancestor {
    my $taxon = shift;
    my $rank = shift || 'species';

    return $taxon if $taxon->rank eq $rank;

RANK: while (1) {
        $taxon = $taxon->ancestor;
        last RANK unless defined $taxon;
        return $taxon if $taxon->rank eq $rank;
    }

    return;
}

__END__
