{%- include "header" -%}
{# Keep a blank line #}
#----------------------------#
# Usage
#----------------------------#
USAGE="
Usage: $0 [STR_IN_FLD] ...

Default values:
    STR_IN_FLD  ''

$ bash cluster.sh Klebsiella Stutzerimonas

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
# Clustering .95 .95
#----------------------------#
# The min sequence identity for clustering
# The min coverage of query and target for clustering
log_info "Clustering .95 .95"
cat species-f.tsv |
    tsv-select -f 2 |
    tsv-uniq |
    parallel --colsep '\t' --no-run-if-empty --linebuffer -k -j {{ parallel2 }} '
        if [[ ! -s {}/pro.fa.gz ]]; then
            exit
        fi
        if [[ -s {}/rep_seq.fa.gz ]]; then
            exit
        fi

        log_debug "{}"

        #cluster-representative cluster-member
        mmseqs easy-cluster "{}"/pro.fa.gz "{}"/rep tmp \
            --threads 4 --remove-tmp-files -v 0 \
            --min-seq-id 0.95 -c 0.95

        rm "{}"/rep_all_seqs.fasta
        hnsm gz "{}"/rep_rep_seq.fasta -o "{}"/rep_seq.fa
        rm "{}"/rep_rep_seq.fasta
    '

#----------------------------#
# Clustering .8 .8
#----------------------------#
log_info "Clustering .8 .8"
cat species-f.tsv |
    tsv-select -f 2 |
    tsv-uniq |
    parallel --colsep '\t' --no-run-if-empty --linebuffer -k -j {{ parallel2 }} '
        if [[ ! -s {}/pro.fa.gz ]]; then
            exit
        fi
        if [[ ! -s {}/rep_seq.fa.gz ]]; then
            exit
        fi
        if [[ -s {}/f88_cluster.tsv ]]; then
            exit
        fi

        log_debug "{}"

        #cluster-representative cluster-member
        mmseqs easy-cluster "{}"/rep_seq.fa.gz "{}"/f88 tmp \
            --threads 4 --remove-tmp-files -v 0 \
            --min-seq-id 0.8 -c 0.8

        rm "{}"/f88_all_seqs.fasta
        rm "{}"/f88_rep_seq.fasta
    '

#----------------------------#
# Clustering .3 .8
#----------------------------#
log_info "Clustering .3 .8"
cat species-f.tsv |
    tsv-select -f 2 |
    tsv-uniq |
    parallel --colsep '\t' --no-run-if-empty --linebuffer -k -j {{ parallel2 }} '
        if [[ ! -s {}/pro.fa.gz ]]; then
            exit
        fi
        if [[ ! -s {}/rep_seq.fa.gz ]]; then
            exit
        fi
        if [[ -s {}/f38_cluster.tsv ]]; then
            exit
        fi

        log_debug "{}"

        #cluster-representative cluster-member
        mmseqs easy-cluster "{}"/rep_seq.fa.gz "{}"/f88 tmp \
            --threads 4 --remove-tmp-files -v 0 \
            --min-seq-id 0.3 -c 0.8

        rm "{}"/f38_all_seqs.fasta
        rm "{}"/f38_rep_seq.fasta
    '

log_info Done.

exit 0
