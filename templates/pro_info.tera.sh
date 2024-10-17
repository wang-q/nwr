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
        tsv-uniq
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
    tsv-uniq |
while read SPECIES; do
    if [[ -f "${SPECIES}"/seq.sqlite ]]; then
        continue
    fi

    if [[ ! -s "${SPECIES}"/res_cluster.tsv ]]; then
        continue
    fi

    log_debug "${SPECIES}"

    nwr seqdb -d ${SPECIES} --init --strain

    nwr seqdb -d ${SPECIES} \
        --size <(
            hnsm size ${SPECIES}/pro.fa.gz
        ) \
        --clust

    cat "${SPECIES}"/strains.tsv |
        parallel --colsep '\t' --no-run-if-empty --linebuffer -k -j 1 '
            if [[ ! -d "../ASSEMBLY/{2}/{1}" ]]; then
                exit
            fi

            gzip -dcf ../ASSEMBLY/{2}/{1}/*_protein.faa.gz |
                grep "^>" |
                sed "s/^>//" |
                sed "s/'\''//g" |
                sed "s/\-\-//g" |
                perl -nl -e '\'' /\[.+\[/ and s/\[/\(/; print; '\'' `#replace [ with ( if there are two consecutive [` |
                perl -nl -e '\'' /\].+\]/ and s/\]/\)/; print; '\'' |
                perl -nl -e '\'' s/\s+\[.+?\]$//g; print; '\'' |
                sed "s/MULTISPECIES: //g" |
                perl -nl -e '\''
                    /^(\w+)\.(\d+)\s+(.+)$/ or next;
                    printf qq(%s.%s\t%s\t%s\n), $1, $2, qq({1}), $3;
                '\''
        ' \
        > "${SPECIES}"/detail.tsv

    nwr seqdb -d ${SPECIES} \
        --anno <(
            tsv-select -f 1,3 "${SPECIES}"/detail.tsv | tsv-uniq
        ) \
        --asmseq <(
            tsv-select -f 1,2 "${SPECIES}"/detail.tsv | tsv-uniq
        )

    rm "${SPECIES}"/detail.tsv

done

log_info Done.

exit 0
