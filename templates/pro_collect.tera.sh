{%- include "header" -%}
{# Keep a blank line #}
#----------------------------#
# Usage
#----------------------------#
USAGE="
Usage: $0 [STR_IN_FLD] ...

Default values:
    STR_IN_FLD  ''

$ bash collect.sh Klebsiella Stutzerimonas

"

if [ "$1" == "-h" ] || [ "$1" == "--help" ]; then
    echo $USAGE
    exit 0
fi

#----------------------------#
# Run
#----------------------------#
log_warn Protein/collect.sh

#----------------------------#
# filtered species.tsv
#----------------------------#
log_info "Protein/species-f.tsv"
cat species.tsv |
    sort |
    if [ "$#" -gt 0 ]; then
        # Initialize an string to store the cmd
        result="tva filter --or"

        # Iterate over each argument and prepend the fixed string
        for arg in "$@"; do
            result+=" --str-in-fld '2:$arg'"
        done

        # Remove the trailing space from the result string
        result=${result% }

        # Execute the result string as a Bash command
        eval "$result"
    else
        tva uniq
    fi |
{% for i in ins -%}
    tva join -f ../{{ i }} -k 1 |
{% endfor -%}
{% for i in not_ins -%}
    tva join -e -f ../{{ i }} -k 1 |
{% endfor -%}
    cat \
    > species-f.tsv

#----------------------------#
# Unique proteins
#----------------------------#
log_info "Unique proteins"
cat species-f.tsv |
    tva select -f 2 |
    tva uniq |
while read SPECIES; do
    if [[ -s "${SPECIES}"/pro.fa.gz ]]; then
        continue
    fi

    log_debug "${SPECIES}"
    mkdir -p "${SPECIES}"

    cat species-f.tsv |
        tva filter --str-eq "2:${SPECIES}" \
        > "${SPECIES}"/strains.tsv

    rm -f "${SPECIES}"/detail.tsv
    rm -f "${SPECIES}"/detail.tsv.gz
    cat "${SPECIES}"/strains.tsv |
        parallel --colsep '\t' --no-run-if-empty --linebuffer -k -j 1 '
            if [[ ! -d "../ASSEMBLY/{2}/{1}" ]]; then
                exit
            fi

            # >WP_011278454.1 biotin--[acetyl-CoA-carboxylase] ligase [Sulfolobus acidocaldarius]
            gzip -dcf ../ASSEMBLY/{2}/{1}/*_protein.faa.gz |
                grep "^>" |
                sed "s/^>//" |
                sed "s/'\''//g" |
                sed "s/\-\-//g" |
                perl -nl -e '\'' s/\s+\[.+?\]$//g; print; '\'' |
                sed "s/MULTISPECIES: //g" |
                perl -nl -e '\''
                    /^(\w+)\.(\d+)\s+(.+)$/ or next;
                    printf qq(%s.%s\t%s\t%s\n), $1, $2, qq({1}), $3;
                '\'' \
                >> {2}/detail.tsv

            gzip -dcf ../ASSEMBLY/{2}/{1}/*_protein.faa.gz
        ' |
        pgr fa filter stdin -u |
        pgr fa gz stdin -p 4 -o "${SPECIES}"/pro.fa.gz

    tva select -f 1,3 "${SPECIES}"/detail.tsv | tva uniq | gzip > "${SPECIES}"/anno.tsv.gz
    tva select -f 1,2 "${SPECIES}"/detail.tsv | tva uniq | gzip > "${SPECIES}"/asmseq.tsv.gz
    rm -f "${SPECIES}"/detail.tsv

done

log_info Done.

exit 0
