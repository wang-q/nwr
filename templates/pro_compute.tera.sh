{%- include "header" -%}
{# Keep a blank line #}
#----------------------------#
# Run
#----------------------------#
log_warn compute.sh

{% set parallel2 = parallel | int / 2 -%}
{% set parallel2 = parallel2 | round(method="floor") -%}
{% if parallel2 < 2 %}{% set parallel2 = 2 %}{% endif -%}

cat species-f.tsv |
    tsv-select -f 2 |
    tsv-uniq |
    parallel --colsep '\t' --no-run-if-empty --linebuffer -k -j {{ parallel2 }} '
        if [[ -f {}/res_rep_seq.fasta.gz ]]; then
            exit
        fi

        log_info "{}"

        #cluster-representative cluster-member
        mmseqs easy-cluster "{}"/pro.fa.gz "{}"/res tmp \
            --threads 2 --remove-tmp-files -v 0 \
            --min-seq-id {{ pro_clust_id }} -c {{ pro_clust_cov }}

        rm "{}"/res_all_seqs.fasta
        pigz -p4 "{}"/res_rep_seq.fasta
    '

log_info Done.

exit 0
