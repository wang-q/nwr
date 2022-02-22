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
use AlignDB::IntSpan;

#----------------------------------------------------------#
# GetOpt section
#----------------------------------------------------------#
my $Config = Config::Tiny->read("$FindBin::RealBin/../config.ini");

=head1 NAME

id_restrict.pl - Restrict taxonomy ids to descendants of an id

=head1 SYNOPSIS

    cat <file> | perl id_restrict.pl [options]
      Options:
        --help      -?          brief help message
        --ancestor  -a  INT     the taxon id of ancestor
        --column    -c  INT     column order where ids are, start from 1
        --separator -s  STR     separator of the line, default is "\s+"

=head1 EXAMPLE

    $ echo 9606 | perl taxon/id_restrict.pl
    9606

    $ echo -e 'Human,9606\nYeast,559292'  | perl taxon/id_restrict.pl -c 2 -s ","
    Human,9606

=cut

# running options
my $td_dir = path( $Config->{path}{td} )->stringify;    # taxdmp

GetOptions(
    'help|?'        => sub { Getopt::Long::HelpMessage(0) },
    'ancestor|a=i'  => \( my $ancestor = 7742 ),               # Vertebrata
    'column|c=i'    => \( my $column = 1 ),
    'separator|s=s' => \( my $separator = '\s+' ),
) or Getopt::Long::HelpMessage(1);

#----------------------------------------------------------#
# init
#----------------------------------------------------------#
$|++;

my $id_set = AlignDB::IntSpan->new;
{
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

    my $taxon_ancestor = $taxon_db->get_taxon( -taxonid => $ancestor );

    my @taxa = $taxon_db->get_all_Descendents($taxon_ancestor);
    my @taxon_ids = map { $_->id() } @taxa;

    #    warn YAML::Syck::Dump \@taxon_ids;

    $id_set->add(@taxon_ids);
}

#----------------------------------------------------------#
# start
#----------------------------------------------------------#
while ( my $line = <> ) {
    chomp $line;
    my @row = split /$separator/, $line;

    my $taxon_id = $row[ $column - 1 ];

    if ( isint($taxon_id) ) {
        if ( $id_set->contains($taxon_id) ) {
            print $line, "\n";
        }
    }
    else {
        warn "[$taxon_id] doesn't look like a taxonmy id.\n";
    }
}

exit;

__END__
