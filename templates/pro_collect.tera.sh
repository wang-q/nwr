{%- include "header" -%}
{# Keep a blank line #}
#----------------------------#
# Run
#----------------------------#
log_warn Protein/collect.sh

CLUST_ID={{ pro_clust_id }}
CLUST_COV={{ pro_clust_cov }}

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
    head -n 10 \
    > species-f.tsv

#----------------------------#
# Unique and representative proteins
#----------------------------#
log_info "Unique and representative proteins"
cat species-f.tsv |
    tsv-select -f 2 |
    tsv-uniq |
while read SPECIES; do
    if [[ -f "${SPECIES}"/res_rep_seq.fasta.gz ]]; then
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
        faops filter -u stdin stdout |
        pigz -p4 \
        > "${SPECIES}"/pro.fa.gz

    #cluster-representative cluster-member
    mmseqs easy-cluster "${SPECIES}"/pro.fa.gz "${SPECIES}"/res tmp \
        --min-seq-id ${CLUST_ID} -c ${CLUST_COV} --remove-tmp-files -v 0

    rm "${SPECIES}"/res_all_seqs.fasta
    pigz -p4 "${SPECIES}"/res_rep_seq.fasta
done

#----------------------------#
# replace.fa
#----------------------------#
log_info "Replacing headers"
cat species-f.tsv |
    tsv-select -f 2 |
    tsv-uniq |
while read SPECIES; do
    if [[ -f "${SPECIES}"/replace.fa.gz ]]; then
        continue
    fi

    rm -f {2}/replace.tsv

    cat "${SPECIES}"/strains.tsv |
        parallel --colsep '\t' --no-run-if-empty --linebuffer -k -j 1 '
            if [[ ! -d "../ASSEMBLY/{2}/{1}" ]]; then
                exit
            fi

            gzip -dcf ../ASSEMBLY/{2}/{1}/*_protein.faa.gz |
                grep "^>" |
                cut -d" " -f 1 |
                sed "s/^>//" |
                perl -nl -e '\''
                    $n = $_;
                    $s = $n;
                    $s =~ s/\.\d+//;
                    printf qq(%s\t%s_%s\t%s\n), $n, {1}, $s, {1};
                '\'' \
                > {2}/{1}.replace.tsv

            cat {2}/{1}.replace.tsv >> {2}/replace.tsv

            faops replace -s \
                ../ASSEMBLY/{2}/{1}/*_protein.faa.gz \
                <(cut -f 1,2 {2}/{1}.replace.tsv) \
                stdout |
                pigz -p4 \
                >> {2}/replace.fa.gz

            rm {2}/{1}.replace.tsv
        '
done

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

    echo -e "#name\tstrain" > "${SPECIES}"/temp.strain.tsv
    cut -f 2,3 "${SPECIES}"/replace.tsv >> "${SPECIES}"/temp.strain.tsv

    echo -e "#name\tsize" > "${SPECIES}"/temp.sizes.tsv
    faops size "${SPECIES}"/replace.fa.gz >> "${SPECIES}"/temp.sizes.tsv

    echo -e "#name\tannotation" > "${SPECIES}"/temp.anno.tsv
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
                    /^(\w+)\.\d+\s+(.+)$/ or next;
                    printf qq(%s_%s\t%s\n), {1}, $1, $2;
                '\''
        ' \
        >> "${SPECIES}"/temp.anno.tsv

    tsv-join \
        "${SPECIES}"/temp.strain.tsv \
        --data-fields 1 \
        -f "${SPECIES}"/temp.sizes.tsv \
        --key-fields 1 \
        --append-fields 2 |
    tsv-join \
        --data-fields 1 \
        -f all.annotation.tsv \
        --key-fields 1 \
        --append-fields 2 \
        > "${SPECIES}"/info.tsv

done

##----------------------------#
## Counts
##----------------------------#
#log_info "Counts"
#
#printf "#item\tcount\n" \
#    > counts.tsv
#
#gzip -dcf all.pro.fa.gz |
#    grep "^>" |
#    wc -l |
#    perl -nl -MNumber::Format -e '
#        printf qq(Proteins\t%s\n), Number::Format::format_number($_, 0,);
#        ' \
#    >> counts.tsv
#
#gzip -dcf all.pro.fa.gz |
#    grep "^>" |
#    tsv-uniq |
#    wc -l |
#    perl -nl -MNumber::Format -e '
#        printf qq(Unique headers and annotations\t%s\n), Number::Format::format_number($_, 0,);
#        ' \
#    >> counts.tsv
#
#gzip -dcf all.uniq.fa.gz |
#    grep "^>" |
#    wc -l |
#    perl -nl -MNumber::Format -e '
#        printf qq(Unique proteins\t%s\n), Number::Format::format_number($_, 0,);
#        ' \
#    >> counts.tsv
#
#gzip -dcf all.replace.fa.gz |
#    grep "^>" |
#    wc -l |
#    perl -nl -MNumber::Format -e '
#        printf qq(all.replace.fa\t%s\n), Number::Format::format_number($_, 0,);
#        ' \
#    >> counts.tsv
#
#cat all.annotation.tsv |
#    wc -l |
#    perl -nl -MNumber::Format -e '
#        printf qq(all.annotation.tsv\t%s\n), Number::Format::format_number($_, 0,);
#        ' \
#    >> counts.tsv
#
#cat all.info.tsv |
#    wc -l |
#    perl -nl -MNumber::Format -e '
#        printf qq(all.info.tsv\t%s\n), Number::Format::format_number($_, 0,);
#        ' \
#    >> counts.tsv

log_info Done.

exit 0
