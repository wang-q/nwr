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
    if [[ ! -s "${SPECIES}/mash.dist.tsv" ]]; then
        continue
    fi

    D_MAX=$(
        cat "${SPECIES}/mash.dist.tsv" |
            tsv-summarize --max 3
    )
    if (( $(echo "$D_MAX < $ANI_VALUE" | bc -l) )); then
        continue
    fi

    # "Link assemblies with the median ANI"
    D_MEDIAN=$(
        cat "${SPECIES}/mash.dist.tsv" |
            tsv-filter --lt "3:$ANI_VALUE" |
            tsv-summarize --median 3
    )
    cat "${SPECIES}/mash.dist.tsv" |
        tsv-filter --ff-str-ne 1:2 --le "3:$D_MEDIAN" |
        hnsm cluster stdin --mode cc |
        tr '\t' '\n' \
        > "${SPECIES}/median.cc.lst"

    log_info "${SPECIES}\t${D_MEDIAN}\t${D_MAX}"
    cat ${SPECIES}/assembly.lst |
        grep -v -Fw -f "${SPECIES}/median.cc.lst"
done |
    tee abnormal.lst

log_info Done.

exit 0
