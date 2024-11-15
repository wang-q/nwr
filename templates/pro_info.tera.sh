{%- include "header" -%}
{# Keep a blank line #}
#----------------------------#
# Usage
#----------------------------#
USAGE="
Usage: $0 [STR_IN_FLD] ...

Default values:
    STR_IN_FLD  ''

$ bash info.sh Klebsiella Stutzerimonas

"

if [ "$1" == "-h" ] || [ "$1" == "--help" ]; then
    echo $USAGE
    exit 0
fi

#----------------------------#
# Run
#----------------------------#
log_warn Protein/info.sh

#----------------------------#
# filtered species.tsv
#----------------------------#
log_info "Protein/species-f.tsv"
cat species.tsv |
    sort |
    if [ "$#" -gt 0 ]; then
        # Initialize an string to store the cmd
        result="tsv-filter --or"

        # Iterate over each argument and prepend the fixed string
        for arg in "$@"; do
            result+=" --str-in-fld '2:$arg'"
        done

        # Remove the trailing space from the result string
        result=${result% }

        # Execute the result string as a Bash command
        eval "$result"
    else
        rgr dedup stdin
    fi |
{% for i in ins -%}
    tsv-join -f ../{{ i }} -k 1 |
{% endfor -%}
{% for i in not_ins -%}
    tsv-join -e -f ../{{ i }} -k 1 |
{% endfor -%}
    cat \
    > species-f.tsv

#----------------------------#
# seq.sqlite
#----------------------------#
log_info "seq.sqlite"
cat species-f.tsv |
    tsv-select -f 2 |
    rgr dedup stdin |
while read SPECIES; do
    if [[ -f "${SPECIES}"/seq.sqlite ]]; then
        continue
    fi

    if [[ ! -s "${SPECIES}"/rep_cluster.tsv ]]; then
        continue
    fi

    log_debug "${SPECIES}"

    nwr seqdb -d ${SPECIES} --init --strain

    nwr seqdb -d ${SPECIES} \
        --size <(
            hnsm size ${SPECIES}/pro.fa.gz
        ) \
        --clust

    nwr seqdb -d ${SPECIES} \
        --anno <(
            gzip -dcf "${SPECIES}"/anno.tsv.gz
        ) \
        --asmseq <(
            gzip -dcf "${SPECIES}"/asmseq.tsv.gz
        )

    nwr seqdb -d ${SPECIES} --rep f1="${SPECIES}"/fam88_cluster.tsv
    nwr seqdb -d ${SPECIES} --rep f2="${SPECIES}"/fam38_cluster.tsv

done

log_info Done.

exit 0
