{%- include "header" -%}
{# Keep a blank line #}
#----------------------------#
# Run
#----------------------------#
log_warn finish.sh

log_info "Strains without protein annotations"
cat url.tsv |
    parallel --colsep '\t' --no-run-if-empty --linebuffer -k -j 4 '
        if ! compgen -G "{3}/{1}/*_protein.faa.gz" > /dev/null; then
            echo {1}
        fi
        if ! compgen -G "{3}/{1}/*_cds_from_genomic.fna.gz" > /dev/null; then
            echo {1}
        fi
    ' |
    tsv-uniq \
    > omit.lst

log_info "ASMs passes the N50 check"
cat collect.tsv |
    tsv-join \
        -H --key-fields 1 \
        --filter-file n50.pass.tsv \
        --append-fields N50,C |
    tsv-join \
        -H --key-fields 1 \
        --filter-file <(
            cat omit.lst |
                sed 's/$/\tNo/' |
                (echo -e "name\tannotations" && cat)
        ) \
        --append-fields annotations --write-all "Yes" \
    > collect.pass.tsv

cat "collect.pass.tsv" |
    sed '1d' |
    tsv-select -f 1 \
    > pass.lst

log_info "Representative or reference strains"
cat collect.pass.tsv |
    tsv-filter -H --not-empty "RefSeq_category" |
    tsv-select -f 1 |
    sed '1d' \
    > rep.lst

log_info "Counts of lines"
printf "#item\tfields\tlines\n" \
    > counts.tsv

for FILE in \
    url.tsv check.lst collect.tsv \
    n50.tsv n50.pass.tsv \
    collect.pass.tsv pass.lst \
    omit.lst rep.lst \
    ; do
    cat ${FILE} |
        datamash check |
        FILE=${FILE} perl -nl -MNumber::Format -e '
            m/(\d+)\s*lines?.+?(\d+)\s*fields?/ or next;
            printf qq($ENV{FILE}\t%s\t%s\n),
                Number::Format::format_number($2, 0,),
                Number::Format::format_number($1, 0,);
            ' \
        >> counts.tsv
done

log_info Done.

exit 0
