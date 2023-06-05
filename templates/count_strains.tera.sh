{%- include "header" -%}
{# Keep a blank line #}
#----------------------------#
# Run
#----------------------------#
log_warn strains.sh

log_info "strains.taxon.tsv"
cat ../ASSEMBLY/collect.pass.csv |
    sed -e '1d' |
    tr "," "\t" |
    tsv-select -f 1,3 |
    nwr append stdin -c 2 -r species -r genus -r family -r order -r class \
    > strains.taxon.tsv

log_info Done.

exit 0
