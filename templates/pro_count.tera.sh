{%- include "header" -%}
{# Keep a blank line #}
#----------------------------#
# Run
#----------------------------#
log_warn Protein/count.sh

#----------------------------#
# filtered species.tsv
#----------------------------#
log_info "Protein/species-f.tsv"
cat species.tsv |
{% for i in ins -%}
    tsv-join -f ../{{ i }} -k 1 |
{% endfor -%}
{% for i in not_ins -%}
    tsv-join -e -f ../{{ i }} -k 1 |
{% endfor -%}
    cat \
    > species-f.tsv

#----------------------------#
# Each species
#----------------------------#
log_info "Count each species"
cat species-f.tsv |
    tsv-select -f 2 |
    tsv-uniq |
while read SPECIES; do
    if [[ -f "${SPECIES}"/counts.tsv ]]; then
        continue
    fi

    if [[ ! -f "${SPECIES}"/seq.sqlite ]]; then
        continue
    fi

    N_STRAIN=$(cat "${SPECIES}"/strains.tsv | wc -l)
    N_TOTAL=$(
        cat "${SPECIES}"/info.tsv |
            tsv-summarize -H --count |
            sed '1d'
        )
    N_DEDUP=$(
        cat "${SPECIES}"/info.tsv |
            tsv-summarize -H --unique-count id |
            sed '1d'
        )
    N_REP=$(
        cat "${SPECIES}"/info.tsv |
            tsv-summarize -H --unique-count rep |
            sed '1d'
        )

    printf "#item\tcount\n" \
        > "${SPECIES}"/counts.tsv

    printf "strain\t%s\n" "${N_STRAIN}" \
        >> "${SPECIES}"/counts.tsv

    printf "total\t%s\n" "${N_TOTAL}" \
        >> "${SPECIES}"/counts.tsv

    printf "dedup\t%s\n" "${N_DEDUP}" \
        >> "${SPECIES}"/counts.tsv

    printf "rep\t%s\n" "${N_REP}" \
        >> "${SPECIES}"/counts.tsv

done

#----------------------------#
# Total
#----------------------------#
log_info "Count total"
cat species-f.tsv |
    tsv-select -f 2 |
    tsv-uniq |
while read SPECIES; do
    if [[ ! -f "${SPECIES}"/counts.tsv ]]; then
        continue
    fi

    cat "${SPECIES}"/counts.tsv |
        datamash transpose |
        sed "s/^count/${SPECIES}/"
done |
    tsv-uniq \
    > counts.tsv

log_info Done.

exit 0
