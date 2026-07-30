#!/usr/bin/perl
use strict;
use warnings;
use autodie;

use Getopt::Long qw();

#----------------------------------------------------------#
# GetOpt section
#----------------------------------------------------------#

=head1 NAME

abbr_name.pl - Abbreviate strain scientific names.

=head1 SYNOPSIS

    cat <file> | perl abbr_name.pl [options]
      Options:
        --help              brief help message
        --column    -c  STR Columns of strain, species, genus, default is 1,2,3.
                            If there's no strain, use 1,1,2.
                            Don't need the strain part, use 2,2,3
                            When there's only strain, use 1,1,1
        --separator -s  STR separator of the line, default is "\s+"
        --min INT           mininal length for abbreviation of species
        --tight             no underscore between Genus and species
        --shortsub          clean subspecies parts

=head1 EXAMPLE

    $ echo -e 'Homo sapiens,Homo\nHomo erectus,Homo\n' |
        perl abbr_name.pl -s ',' -c "1,1,2"
    Homo sapiens,Homo,H_sap
    Homo erectus,Homo,H_ere

    $ echo -e 'Homo sapiens,Homo\nHomo erectus,Homo\n' |
        perl abbr_name.pl -s ',' -c "1,1,2" --tight
    Homo sapiens,Homo,Hsap
    Homo erectus,Homo,Here

    $ echo -e 'Homo sapiens,Homo\nHomo erectus,Homo\n' |
        perl abbr_name.pl -s ',' -c "1,2,2"
    Homo sapiens,Homo,H_sap
    Homo erectus,Homo,H_ere

    $ echo -e 'Homo sapiens sapiens,Homo sapiens,Homo\nHomo erectus,Homo erectus,Homo\n' |
        perl abbr_name.pl -s ',' -c "1,2,3" --tight
    Homo sapiens sapiens,Homo sapiens,Homo,Hsap_sapiens
    Homo erectus,Homo erectus,Homo,Here

    $ echo -e 'Homo\nHomo\nGorilla\n' |
        perl abbr_name.pl -s ',' -c "1,1,1"
    Homo,H
    Homo,H
    Gorilla,G

    $ echo -e 'Homo sapiens,Homo\nCandida albicans,Candida\n[Candida] auris,[Candida]\n[Candida] haemuloni,Candida/Metschnikowiaceae\n[Candida] boidinii,Ogataea\n' |
        perl abbr_name.pl -s ',' -c "1,1,2"
    Homo sapiens,Homo,H_sap
    Candida albicans,Candida,C_alb
    [Candida] auris,[Candida],Candida_auris
    [Candida] haemuloni,Candida/Metschnikowiaceae,Candida_haemuloni
    [Candida] boidinii,Ogataea,Candida_boidinii

    $ echo -e 'Legionella pneumophila subsp. pneumophila str. Philadelphia 1\nLeptospira interrogans serovar Copenhageni str. Fiocruz L1-130\n' |
        perl abbr_name.pl -s ',' -m 0 -c "1,1,1" --shortsub
    Legionella pneumophila subsp. pneumophila str. Philadelphia 1,Leg_pneumophila_pneumophila_Philadelphia_1
    Leptospira interrogans serovar Copenhageni str. Fiocruz L1-130,Lep_interrogans_Copenhageni_Fiocruz_L1_130

=cut

Getopt::Long::GetOptions(
    'help|?'        => sub { Getopt::Long::HelpMessage(0) },
    'column|c=s'    => \( my $column      = '1,2,3' ),
    'separator|s=s' => \( my $separator   = '\s+' ),
    'min|m=i'       => \( my $min_species = 3 ),
    'tight'         => \my $tight,
    'shortsub'      => \my $shortsub,
) or Getopt::Long::HelpMessage(1);

#----------------------------------------------------------#
# init
#----------------------------------------------------------#
$|++;

#----------------------------------------------------------#
# start
#----------------------------------------------------------#
my @columns = map { $_ - 1 } split( /,/, $column );
my @fields;
my @rows;
while ( my $line = <> ) {
    chomp $line;
    next if $line eq '';
    my @row = split /$separator/, $line;
    s/"|'//g for @row;

    my ( $strain, $species, $genus ) = @row[@columns];
    my $is_normal = 0;

    if ( $genus ne $species ) {

        # not like [Candida]
        if ( $genus =~ /^\w/ ) {

            # $species starts with $genus
            if ( rindex( $species, $genus, 0 ) == 0 ) {

                # $strain starts with $species
                if ( rindex( $strain, $species, 0 ) == 0 ) {
                    $strain =~ s/^\Q$species\E ?//;

                    $species =~ s/^\Q$genus\E //;

                    $is_normal = 1;
                }
            }
        }
    }

    # do not abbr species parts
    else {
        if ( $genus =~ /^\w/ ) {
            if ( rindex( $strain, $genus, 0 ) == 0 ) {
                $strain =~ s/^\Q$genus\E ?//;
                $species   = '';
                $is_normal = 1;
            }
        }
    }

    # Remove `Candidatus`
    $genus =~ s/\bCandidatus \b/C/gi;

    # Clean long subspecies names
    if ( defined $shortsub ) {
        $strain =~ s/\bsubsp\b//gi;
        $strain =~ s/\bserovar\b//gi;
        $strain =~ s/\bstr\b//gi;
        $strain =~ s/\bstrain\b//gi;
        $strain =~ s/\bsubstr\b//gi;
        $strain =~ s/\bserotype\b//gi;
        $strain =~ s/\bbiovar\b//gi;
        $strain =~ s/\bvar\b//gi;
        $strain =~ s/\bgroup\b//gi;
        $strain =~ s/\bvariant\b//gi;
        $strain =~ s/\bgenomovar\b//gi;
        $strain =~ s/\bgenomosp\b//gi;

        $strain =~ s/\bbreed\b//gi;
        $strain =~ s/\bcultivar\b//gi;
        $strain =~ s/\becotype\b//gi;

        $strain =~ s/\bn\/a\b//;
        $strain =~ s/\bNA\b//;

        $strain =~ s/\bmicrobial\b//gi;
        $strain =~ s/\bclinical\b//gi;
        $strain =~ s/\bpathogenic\b//gi;
        $strain =~ s/\bisolate\b//gi;
    }

    s/\W+/_/g for ( $strain, $species, $genus );
    s/_+/_/g  for ( $strain, $species, $genus );
    s/_$//    for ( $strain, $species, $genus );
    s/^_//    for ( $strain, $species, $genus );

    push @fields, [ $strain, $species, $genus, $is_normal ];

    push @rows, \@row;
}

my $count = scalar @fields;

my @ge = map { $_->[3] ? $_->[2] : () } @fields;
my @sp = map { $_->[3] ? $_->[1] : () } @fields;

my $ge_of = abbr_most( [ uniq(@ge) ], 1,            "Yes" );
my $sp_of = abbr_most( [ uniq(@sp) ], $min_species, "Yes" );

for my $i ( 0 .. $count - 1 ) {
    if ( $fields[$i]->[3] ) {

        my $spacer = $tight ? '' : '_';
        my $ge_sp  = join $spacer,
          grep { defined $_ and length $_ } $ge_of->{ $fields[$i]->[2] },
          $sp_of->{ $fields[$i]->[1] };
        my $organism = join "_", grep { defined $_ and length $_ } $ge_sp,
          $fields[$i]->[0];

        print join( ",", @{ $rows[$i] }, $organism ), "\n";
    }
    else {
        print join( ",", @{ $rows[$i] }, $fields[$i]->[0] ), "\n";
    }
}

exit;

sub uniq {
    my %seen = ();
    my $k;
    my $seen_undef;
    return grep { defined $_ ? not $seen{ $k = $_ }++ : not $seen_undef++ } @_;
}

# from core module Text::Abbrev
sub abbr {
    my $list = shift;
    my $min  = shift;
    return {} unless @{$list};

    $min = 1 unless $min;
    my $hashref = {};
    my %table;

  WORD: for my $word ( @{$list} ) {
        for ( my $len = length($word) - 1 ; $len >= $min ; --$len ) {
            my $abbrev = substr( $word, 0, $len );
            my $seen   = ++$table{$abbrev};
            if ( $seen == 1 ) {    # We're the first word so far to have
                                   # this abbreviation.
                $hashref->{$abbrev} = $word;
            }
            elsif ( $seen == 2 ) {    # We're the second word to have this
                                      # abbreviation, so we can't use it.
                delete $hashref->{$abbrev};
            }
            else {                    # We're the third word to have this
                                      # abbreviation, so skip to the next word.
                next WORD;
            }
        }
    }

    # Non-abbreviations always get entered, even if they aren't unique
    for my $word ( @{$list} ) {
        $hashref->{$word} = $word;
    }
    return $hashref;
}

sub abbr_most {
    my $list  = shift;
    my $min   = shift;
    my $creat = shift; # don't abbreviate 1 letter. "I'd spell creat with an e."
    return {} unless @{$list};

    # don't abbreviate
    if ( defined $min and $min == 0 ) {
        my %abbr_of;
        for my $word ( @{$list} ) {
            $abbr_of{$word} = $word;
        }
        return \%abbr_of;
    }

    # do abbreviate
    else {
        my $hashref = $min ? abbr( $list, $min ) : abbr($list);
        my @ks      = sort keys %{$hashref};
        my %abbr_of;
        for my $i ( reverse( 0 .. $#ks ) ) {
            if ( index( $ks[$i], $ks[ $i - 1 ] ) != 0 ) {
                $abbr_of{ $hashref->{ $ks[$i] } } = $ks[$i];
            }

            if ($creat) {
                for my $key ( keys %abbr_of ) {
                    if ( length($key) - length( $abbr_of{$key} ) == 1 ) {
                        $abbr_of{$key} = $key;
                    }
                }
            }
        }

        return \%abbr_of;
    }
}

__END__
