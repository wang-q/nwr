{%- include "header" -%}
{# Keep a blank line #}
#----------------------------#
# Run
#----------------------------#
log_warn Protein/count.sh

#----------------------------#
# Each species
#----------------------------#
log_info "Count each species"
cat species-f.tsv |
    tsv-select -f 2 |
    tsv-uniq |
while read SPECIES; do
    if [[ ! -f "${SPECIES}"/info.tsv ]]; then
        continue
    fi

    N_STRAIN=$(cat "${SPECIES}"/strains.tsv | wc -l)
    N_TOTAL=$(cat "${SPECIES}"/replace.tsv | wc -l)
    N_DEDUP=$(cat "${SPECIES}"/res_cluster.tsv | wc -l)
    N_REP=$(
        cat "${SPECIES}"/res_cluster.tsv |
            tsv-select -f 1 | tsv-uniq |
            wc -l
        )

    printf "#item\tcount\n" \
        > "${SPECIES}"/counts.tsv

    printf "strain\t%s\n" "${N_STRAIN}" \
        >> "${SPECIES}"/counts.tsv

    printf "total\t%s\n" "${N_TOTAL}" \
        >> "${SPECIES}"/counts.tsv

    printf "dedup\t%s\n" "${N_DEDUP}" \
        >> "${SPECIES}"/counts.tsv

    printf "rep\t%s\n" "${N_REP}" \
        >> "${SPECIES}"/counts.tsv

done

#----------------------------#
# Total
#----------------------------#
log_info "Count total"
cat species-f.tsv |
    tsv-select -f 2 |
    tsv-uniq |
while read SPECIES; do
    if [[ ! -f "${SPECIES}"/counts.tsv ]]; then
        continue
    fi

    datamash transpose < "${SPECIES}"/counts.tsv
done |
    tsv-uniq \
    > counts.tsv

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
