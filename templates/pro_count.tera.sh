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
    rgr dedup stdin |
while read SPECIES; do
    if [[ -f "${SPECIES}"/counts.tsv ]]; then
        continue
    fi

    if [[ ! -f "${SPECIES}"/seq.sqlite ]]; then
        continue
    fi

    log_debug "${SPECIES}"

    echo "
.header ON
        SELECT
            '${SPECIES}' AS species,
            COUNT(DISTINCT asm_seq.asm_id) AS strain,
            COUNT(*) AS total,
            COUNT(DISTINCT rep_seq.seq_id) AS dedup,
            COUNT(DISTINCT rep.id) AS rep,
            COUNT(DISTINCT rep.f1) AS fam88,
            COUNT(DISTINCT rep.f2) AS fam38
        FROM asm_seq
        JOIN rep_seq ON asm_seq.seq_id = rep_seq.seq_id
        JOIN rep ON rep_seq.rep_id = rep.id
        WHERE 1=1
        " |
        sqlite3 -tabs ${SPECIES}/seq.sqlite \
        > "${SPECIES}"/counts.tsv

done

#----------------------------#
# Total
#----------------------------#
log_info "Count total"
cat species-f.tsv |
    tsv-select -f 2 |
    rgr dedup stdin |
while read SPECIES; do
    if [[ ! -f "${SPECIES}"/counts.tsv ]]; then
        continue
    fi

    cat "${SPECIES}"/counts.tsv
done |
    rgr dedup stdin \
    > counts.tsv

log_info Done.

exit 0
