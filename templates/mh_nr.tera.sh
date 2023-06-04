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

    echo >&2 "    Scores based on representative.lst, omit.lst, and assembly_level"
    # score.tsv

    cat "${SPECIES}/connected_components.tsv" |
        perl -nla -MPath::Tiny -F"\t" -e '
            BEGIN {
                our %rep = map { ($_, 1) } path( q(../ASSEMBLY/representative.lst) )->lines({chomp => 1});
                our %omit = map { ($_, 1) } path( q(../ASSEMBLY/omit.lst) )->lines({chomp => 1});
            }

            # Representative strains are preferred
            if ( grep { $rep{$_} } @F ) {
                @F = grep { ! $rep{$_} } @F
            }
            else {
                shift @F;
            }
            printf qq(%s\n), $_ for @F;
            ' \
        > "${SPECIES}/redundant.lst"

    cat "${SPECIES}/assembly.lst" |
        tsv-join --exclude -f "${SPECIES}/redundant.lst" \
        > "${SPECIES}/NR.lst"

done

log_info Done.

exit 0
