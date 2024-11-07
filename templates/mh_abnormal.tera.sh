{%- include "header" -%}
{# Keep a blank line #}
#----------------------------#
# Run
#----------------------------#
log_warn abnormal.sh

ANI_VALUE={{ mh_ani_ab }}

log_info Abnormal strains

cat species.lst |
while read SPECIES; do
#    log_debug "${SPECIES}"

    # Number of assemblies >= 2
    N_ASM=$(
        cat "${SPECIES}/assembly.lst" | wc -l
    )
    if [[ $N_ASM -lt 2 ]]; then
        continue
    fi

    # Number of non-redundant >= 2
    N_NR=$(
        cat "${SPECIES}/NR.lst" | wc -l
    )
    if [[ $N_NR -lt 4 ]]; then
        continue
    fi

    if [[ ! -s "${SPECIES}/mash.ab.tsv" ]]; then
        mash triangle -E -p 8 -l <(
            cat "${SPECIES}/NR.lst" |
                parallel --no-run-if-empty --linebuffer -k -j 1 "
                    if [[ -e ${SPECIES}/msh/{}.msh ]]; then
                        echo ${SPECIES}/msh/{}.msh
                    fi
                "
            ) \
            > "${SPECIES}/mash.ab.tsv"
    fi

    D_MAX=$(cat "${SPECIES}/mash.ab.tsv" |
        tsv-summarize --max 3
    )
    if (( $(echo "$D_MAX < $ANI_VALUE" | bc -l) )); then
        continue
    fi

    # "Link assemblies with the median ANI smaller than $ANI_VALUE"
    D_MEDIAN=$(
        cat "${SPECIES}/mash.ab.tsv" |
            tsv-filter --lt "3:$ANI_VALUE" |
            SPECIES=${SPECIES} perl -nla -MPath::Tiny -F"\t" -e '
                BEGIN {
                    # To reduce computation and storage overhead
                    our %red_of = map {
                        my @fields = split(qq(\t), $_);
                        ($fields[0], scalar @fields)
                        }
                        path( qq($ENV{SPECIES}/RED.cc.tsv) )->lines({chomp => 1});
                    for my $asm ( path( qq($ENV{SPECIES}/assembly.lst) )->lines({chomp => 1}) ) {
                        if (not exists $red_of{$asm}) {
                            $red_of{$asm} = 1;
                        }
                    }
                    our %val_count_of;
                }

                my $mash = $F[2];
                my $count = $red_of{$F[0]} * $red_of{$F[1]};
                $val_count_of{$mash} += $count;

                END {
                    my $total;
                    for my $key (keys %val_count_of) {
                        $total += $val_count_of{$key};
                    }
                    my $mid = int($total / 2);
                    my $cur;
                    for my $key (sort keys %val_count_of) {
                        $cur += $val_count_of{$key};
                        if ($cur > $mid) {
                            print $key;
                            last;
                        }
                    }
                }
            '
    )

    if [[ -z "$D_MEDIAN" ]]; then
        log_info "${SPECIES}\t${D_MAX}\tNot enough data"
        continue
    fi

    cat "${SPECIES}/mash.ab.tsv" |
        tsv-filter --ff-str-ne 1:2 --le "3:$D_MEDIAN" |
        hnsm cluster stdin --mode cc |
        tr '\t' '\n' \
        > "${SPECIES}/cc.lst"

    log_info "${SPECIES}\t${D_MAX}\t${D_MEDIAN}"
    cat ${SPECIES}/NR.lst |
        grep -v -Fw -f "${SPECIES}/cc.lst"
done |
    tee abnormal.lst

log_info Done.

exit 0
