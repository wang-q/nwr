{%- include "header" -%}
{# Keep a blank line #}
#----------------------------#
# Run
#----------------------------#
log_warn strains.sh

if [[ -e "../ASSEMBLY/collect.pass.csv" ]]; then
    cat "../ASSEMBLY/collect.pass.csv" |
        sed '1d' |
        tsv-select -d, -f 1
else
    cat species.tsv |
        tsv-select -f 1
fi \
    > pass.lst

log_info "strains.taxon.tsv"
cat species.tsv |
{% if pass == "1" -%}
    tsv-join -f pass.lst -k 1 |
{% endif -%}
{% for i in count_ins -%}
    tsv-join -f ../{{ i }} -k 1 |
{% endfor -%}
{% for i in count_not_ins -%}
    tsv-join -e -f ../{{ i }} -k 1 |
{% endfor -%}
    nwr append stdin -c 2 -r genus -r family -r order -r class \
    > strains.taxon.tsv

log_info Done.

exit 0
