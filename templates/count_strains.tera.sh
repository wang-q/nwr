{%- include "header" -%}
{# Keep a blank line #}
#----------------------------#
# Run
#----------------------------#
log_warn strains.sh

log_info "strains.taxon.tsv"
cat species.tsv |
{% for i in ins -%}
    tva join -f ../{{ i }} -k 1 |
{% endfor -%}
{% for i in not_ins -%}
    tva join -e -f ../{{ i }} -k 1 |
{% endfor -%}
    nwr append stdin -c 2 -r genus -r family -r order -r class \
    > strains.taxon.tsv

log_info "taxa.tsv"
cat strains.taxon.tsv |
    tva stats --unique-count 1-6 |
    (echo -e "strain\tspecies\tgenus\tfamily\torder\tclass" && cat) |
    tva transpose |
    (echo -e "item\tcount" && cat) \
    > taxa.tsv

log_info Done.

exit 0
