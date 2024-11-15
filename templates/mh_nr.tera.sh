{%- include "header" -%}
{# Keep a blank line #}
#----------------------------#
# Run
#----------------------------#
log_warn nr.sh

ANI_VALUE={{ mh_ani_nr }}

log_info Non-redundant strains

cat species.tsv |
{% for i in ins -%}
    tsv-join -f ../{{ i }} -k 1 |
{% endfor -%}
{% for i in not_ins -%}
    tsv-join -e -f ../{{ i }} -k 1 |
{% endfor -%}
    tsv-select -f 2 |
    rgr dedup stdin \
    > species.lst

cat species.lst |
while read SPECIES; do
    log_debug "${SPECIES}"

    if [[ -f "${SPECIES}/NR.lst" ]]; then
        continue
    fi

    mkdir -p "${SPECIES}"

    cat species.tsv |
{% for i in ins -%}
    tsv-join -f ../{{ i }} -k 1 |
{% endfor -%}
{% for i in not_ins -%}
    tsv-join -e -f ../{{ i }} -k 1 |
{% endfor -%}
        tsv-filter --str-eq "2:${SPECIES}" |
        tsv-select -f 1 \
        > "${SPECIES}/assembly.lst"

    # Number of assemblies >= 2
    N_ASM=$(
        cat "${SPECIES}/assembly.lst" | wc -l
    )
    if [[ $N_ASM -lt 2 ]]; then
        continue
    fi

    echo >&2 "    mash distances"
    cat "${SPECIES}/assembly.lst" |
        parallel --no-run-if-empty --linebuffer -k -j 1 "
            if [[ -e ${SPECIES}/msh/{}.msh ]]; then
                echo ${SPECIES}/msh/{}.msh
            fi
        " \
        > "${SPECIES}/msh.lst"

    if [[ ! -s "${SPECIES}/mash.nr.tsv" ]]; then
        mash triangle -E -p 8 -l "${SPECIES}/msh.lst" |
        tsv-filter --ff-str-ne 1:2 --le "3:${ANI_VALUE}" \
            > "${SPECIES}/mash.nr.tsv"
    fi

    echo >&2 "    Finding redundants"
    cat "${SPECIES}/mash.nr.tsv" |
        hnsm cluster stdin --mode cc \
        > "${SPECIES}/RED.cc.tsv"

    echo >&2 "    Scoring based on rep.lst, omit.lst, and assembly_level"
    cat ${SPECIES}/assembly.lst |
        tsv-join -f ../ASSEMBLY/rep.lst -k 1 -a 1 --write-all "0" |
        tsv-join -f ../ASSEMBLY/omit.lst -k 1 -a 1 --write-all "0" |
        tsv-join -f species.tsv -k 1 -a 3 \
        > ${SPECIES}/scores.tsv

    cat "${SPECIES}/RED.cc.tsv" |
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
