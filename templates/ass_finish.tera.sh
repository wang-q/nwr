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

log_info "ASMs passed the N50 check"
tsv-join \
    collect.csv \
    --delimiter "," -H --key-fields 1 \
    --filter-file n50.pass.csv \
    > collect.pass.csv

log_info "Counts of lines"
printf "#item\tcount\n" \
    > counts.tsv

for FILE in url.tsv check.lst collect.csv n50.tsv n50.pass.csv omit.lst collect.pass.csv; do
    cat ${FILE} |
        wc -l |
        FILE=${FILE} perl -nl -MNumber::Format -e '
            printf qq($ENV{FILE}\t%s\n), Number::Format::format_number($_, 0,);
            ' \
        >> counts.tsv
done

log_info Done.

exit 0