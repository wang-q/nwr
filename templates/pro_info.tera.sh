{%- include "header" -%}
{# Keep a blank line #}
#----------------------------#
# Run
#----------------------------#
log_warn compute.sh

#----------------------------#
# info.tsv
#----------------------------#
log_info "info.tsv"
cat species-f.tsv |
    tsv-select -f 2 |
    tsv-uniq |
while read SPECIES; do
    if [[ -f "${SPECIES}"/info.tsv ]]; then
        continue
    fi

    if [[ ! -s "${SPECIES}"/res_cluster.tsv ]]; then
        continue
    fi

    log_debug "${SPECIES}"

    echo -e "#name\tid\tstrain\tannotation" > "${SPECIES}"/temp.strain.tsv
    cat "${SPECIES}"/strains.tsv |
        parallel --colsep '\t' --no-run-if-empty --linebuffer -k -j 1 '
            if [[ ! -d "../ASSEMBLY/{2}/{1}" ]]; then
                exit
            fi

            gzip -dcf ../ASSEMBLY/{2}/{1}/*_protein.faa.gz |
                grep "^>" |
                sed "s/^>//" |
                perl -nl -e '\'' /\[.+\[/ and s/\[/\(/; print; '\'' |
                perl -nl -e '\'' /\].+\]/ and s/\]/\)/; print; '\'' |
                perl -nl -e '\'' s/\s+\[.+?\]$//g; print; '\'' |
                sed "s/MULTISPECIES: //g" |
                perl -nl -e '\''
                    /^(\w+)\.(\d+)\s+(.+)$/ or next;
                    printf qq(%s_%s\t%s.%s\t%s\t%s\n), {1}, $1, $1, $2, {1}, $3;
                '\''
        ' \
        >> "${SPECIES}"/temp.strain.tsv

    echo -e "id\tsize" > "${SPECIES}"/temp.sizes.tsv
    faops size "${SPECIES}"/pro.fa.gz >> "${SPECIES}"/temp.sizes.tsv

    echo -e "rep\tid" > "${SPECIES}"/temp.clust.tsv
    cat "${SPECIES}"/res_cluster.tsv >> "${SPECIES}"/temp.clust.tsv

    #name	id	rep	strain	size	annotation
    tsv-join -H \
        "${SPECIES}"/temp.strain.tsv \
        --data-fields id \
        -f "${SPECIES}"/temp.sizes.tsv \
        --key-fields id \
        --append-fields 2 |
        tsv-join -H \
            --data-fields id \
            -f "${SPECIES}"/temp.clust.tsv \
            --key-fields id \
            --append-fields 1 |
        tsv-select -f 1,2,6,3,5,4 \
        > "${SPECIES}"/info.tsv

    rm -f "${SPECIES}"/temp.*.tsv

done

log_info Done.

exit 0
