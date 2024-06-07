{%- include "header" -%}
{# Keep a blank line #}
#----------------------------#
# Run
#----------------------------#
log_warn Protein/collect.sh

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
    if [[ -s "${SPECIES}"/info.tsv ]]; then
        continue
    fi

    echo -e "#name\tid\tstrain" > "${SPECIES}"/temp.strain.tsv
    tsv-select -f 2,1,3 "${SPECIES}"/replace.tsv >> "${SPECIES}"/temp.strain.tsv

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
        -f "${SPECIES}"/temp.anno.tsv \
        --key-fields 1 \
        --append-fields 2 \
        > "${SPECIES}"/info.tsv

    rm -f "${SPECIES}"/temp.*.tsv

    rm -f "${SPECIES}"/replace.fa.gz
    rm -f "${SPECIES}"/replace.tsv
done

log_info Done.

exit 0
