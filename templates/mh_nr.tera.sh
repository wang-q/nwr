{%- include "header" -%}
{# Keep a blank line #}
#----------------------------#
# Run
#----------------------------#
log_warn nr.sh

ANI_VALUE_THRESHOLD={{ mh_ani_nr }}

log_info Non-redundant strains

cat species.lst |
while read SPECIES; do
    log_debug "${SPECIES}"

    # Number of assemblies >= 2
    if [[ ! -s "${SPECIES}/mash.dist.tsv" ]]; then
        continue
    fi

    echo >&2 "    List NR"
    cat "${SPECIES}/mash.dist.tsv" |
        tsv-filter --ff-str-ne 1:2 --le "3:${ANI_VALUE_THRESHOLD}" \
        > "${SPECIES}/redundant.dist.tsv"

    echo >&2 "    Connected components"
    cat "${SPECIES}/redundant.dist.tsv" |
        perl -nla -F"\t" -MGraph::Undirected -e '
            BEGIN {
                our $g = Graph::Undirected->new;
            }

            $g->add_edge($F[0], $F[1]);

            END {
                for my $cc ( $g->connected_components ) {
                    print join qq{\t}, sort @{$cc};
                }
            }
        ' \
        > "${SPECIES}/connected_components.tsv"

    echo >&2 "    Scores based on rep.lst, omit.lst, and assembly_level"
    cat ${SPECIES}/assembly.lst |
        tsv-join -f ../ASSEMBLY/rep.lst -k 1 -a 1 --write-all "0" |
        tsv-join -f ../ASSEMBLY/omit.lst -k 1 -a 1 --write-all "0" |
        tsv-join -f species.tsv -k 1 -a 3 \
        > ${SPECIES}/scores.tsv

    cat "${SPECIES}/connected_components.tsv" |
        SPECIES=${SPECIES} perl -nla -MPath::Tiny -F"\t" -e '
            BEGIN {
                our %rep_of = map { ($_, 1) } path( q(../ASSEMBLY/rep.lst) )->lines({chomp => 1});
                our %omit_of = map { ($_, 1) } path( q(../ASSEMBLY/omit.lst) )->lines({chomp => 1});
                our %level_of = map { ( split(qq(\t), $_) )[0, 2] } path( q(species.tsv) )->lines({chomp => 1});
            }

            my @sorted = @F;

            # Level of "Complete Genome"/1 and Chromosome/2 are preferred
            @sorted =
                map  { $_->[0] }
                sort { $a->[1] <=> $b->[1] }
                map { [$_, $level_of{$_}] }
                @sorted;

            # With annotations
            @sorted =
                map  { $_->[0] }
                sort { $a->[1] <=> $b->[1] }
                map { [$_, exists $omit_of{$_} ? 1 : 0 ] }
                @sorted;

            # Representative strains
            @sorted =
                map  { $_->[0] }
                sort { $b->[1] <=> $a->[1] }
                map { [$_, exists $rep_of{$_} ? 1 : 0 ] }
                @sorted;

            shift @sorted; # The first is NR
            printf qq(%s\n), $_ for @sorted;
            ' \
        > "${SPECIES}/redundant.lst"

    cat "${SPECIES}/assembly.lst" |
        tsv-join --exclude -f "${SPECIES}/redundant.lst" \
        > "${SPECIES}/NR.lst"

done

log_info Done.

exit 0
