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

{% set parallel2 = parallel | int / 4 -%}
{% set parallel2 = parallel2 | round(method="floor") -%}
{% if parallel2 < 1 %}{% set parallel2 = 1 %}{% endif -%}

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
# Unique proteins
#----------------------------#
log_info "Unique proteins"
cat species-f.tsv |
    tsv-select -f 2 |
    if [ "$#" -gt 0 ]; then
        # Initialize an string to store the cmd
        result="tsv-filter --or"

        # Iterate over each argument and prepend the fixed string
        for arg in "$@"; do
            result+=" --str-in-fld '1:$arg'"
        done

        # Remove the trailing space from the result string
        result=${result% }

        # Execute the result string as a Bash command
        eval "$result"
    else
        tsv-uniq
    fi |
    tsv-uniq |
while read SPECIES; do
    if [[ -s "${SPECIES}"/pro.fa.gz ]]; then
        continue
    fi

    log_debug "${SPECIES}"
    mkdir -p "${SPECIES}"

    cat species-f.tsv |
        tsv-filter --str-eq "2:${SPECIES}" \
        > "${SPECIES}"/strains.tsv

    cat "${SPECIES}"/strains.tsv |
        parallel --colsep '\t' --no-run-if-empty --linebuffer -k -j 1 '
            if [[ ! -d "../ASSEMBLY/{2}/{1}" ]]; then
                exit
            fi

            gzip -dcf ../ASSEMBLY/{2}/{1}/*_protein.faa.gz
        ' |
        hnsm filter stdin -u |
        hnsm gz stdin -p 4 -o "${SPECIES}"/pro.fa

done

#----------------------------#
# Clustering
#----------------------------#
log_info "Clustering"
cat species-f.tsv |
    tsv-select -f 2 |
    if [ "$#" -gt 0 ]; then
        # Initialize an string to store the cmd
        result="tsv-filter --or"

        # Iterate over each argument and prepend the fixed string
        for arg in "$@"; do
            result+=" --str-in-fld '1:$arg'"
        done

        # Remove the trailing space from the result string
        result=${result% }

        # Execute the result string as a Bash command
        eval "$result"
    else
        tsv-uniq
    fi |
    tsv-uniq |
    parallel --colsep '\t' --no-run-if-empty --linebuffer -k -j {{ parallel2 }} '
        if [[ -s {}/rep_seq.fa.gz ]]; then
            exit
        fi

        log_debug "{}"

        #cluster-representative cluster-member
        mmseqs easy-cluster "{}"/pro.fa.gz "{}"/res tmp \
            --threads 4 --remove-tmp-files -v 0 \
            --min-seq-id {{ pro_clust_id }} -c {{ pro_clust_cov }}

        rm "{}"/res_all_seqs.fasta
        pigz -p4 "{}"/res_rep_seq.fasta
    '

log_info Done.

exit 0
