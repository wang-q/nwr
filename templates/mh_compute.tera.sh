{%- include "header" -%}
{# Keep a blank line #}
#----------------------------#
# Run
#----------------------------#
log_warn compute.sh

{% set parallel2 = parallel | int / 2 -%}
{% set parallel2 = parallel2 | round(method="floor") -%}
{% if parallel2 < 2 %}{% set parallel2 = 1 %}{% endif -%}

cat species.tsv |
{% for i in ins -%}
    tsv-join -f ../{{ i }} -k 1 |
{% endfor -%}
{% for i in not_ins -%}
    tsv-join -e -f ../{{ i }} -k 1 |
{% endfor -%}
    parallel --colsep '\t' --no-run-if-empty --linebuffer -k -j {{ parallel2 }} '
        if [[ -s "{2}/msh/{1}.msh" ]]; then
            exit
        fi

        log_info "{2}\t{1}"
        mkdir -p "{2}/msh"

        find ../ASSEMBLY/{2}/{1} -name "*_genomic.fna.gz" |
            grep -v "_from_" |
            xargs gzip -dcf |
            mash sketch -k 21 -s {{ mh_sketch }} -p 2 - -I "{1}" -o "{2}/msh/{1}"
    '

log_info Done.

exit 0
