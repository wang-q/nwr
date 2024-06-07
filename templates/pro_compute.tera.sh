{%- include "header" -%}
{# Keep a blank line #}
#----------------------------#
# Run
#----------------------------#
log_warn compute.sh

{% set parallel2 = parallel | int / 4 -%}
{% set parallel2 = parallel2 | round(method="floor") -%}
{% if parallel2 < 1 %}{% set parallel2 = 1 %}{% endif -%}

#----------------------------#
# clustering
#----------------------------#
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
            --threads 4 --remove-tmp-files -v 0 \
            --min-seq-id {{ pro_clust_id }} -c {{ pro_clust_cov }}

        rm "{}"/res_all_seqs.fasta
        pigz -p4 "{}"/res_rep_seq.fasta
    '

log_info "info.tsv"
cat species-f.tsv |
    tsv-select -f 2 |
    tsv-uniq |
while read SPECIES; do
    if [[ ! -s "${SPECIES}"/res_rep_seq.fasta.gz ]]; then
        continue
    fi

    echo -e "rep\tid" > "${SPECIES}"/temp.clust.tsv
    cat "${SPECIES}"/res_cluster.tsv >> "${SPECIES}"/temp.clust.tsv

    tsv-join \
        "${SPECIES}"/info.tsv \
        -H --data-fields id \
        -f "${SPECIES}"/temp.clust.tsv \
        --key-fields id \
        --append-fields rep \
        > "${SPECIES}"/temp.info.tsv

    tsv-select \
        "${SPECIES}"/temp.info.tsv \
        -f 1,2,6,3-5 \
        > "${SPECIES}"/info.tsv

    rm -f "${SPECIES}"/temp.*.tsv
    rm -f "${SPECIES}"/res_cluster.tsv

done

log_info Done.

exit 0
