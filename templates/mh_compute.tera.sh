{%- include "header" -%}
{# Keep a blank line #}
#----------------------------#
# Run
#----------------------------#
log_warn compute.sh

if [[ -e "../ASSEMBLY/collect.pass.csv" ]]; then
    cat "../ASSEMBLY/collect.pass.csv" |
        sed '1d' |
        tsv-select -d, -f 1
else
    cat species.tsv |
        tsv-select -f 1
fi \
    > pass.lst

cat species.tsv |
{% if pass == "1" -%}
    tsv-join -f pass.lst -k 1 |
{% endif -%}
    parallel --colsep '\t' --no-run-if-empty --linebuffer -k -j 4 '
        if [[ -e "{2}/{1}.msh" ]]; then
            exit
        fi
        log_info "{2}\t{1}"
        mkdir -p "{2}"

        find ../ASSEMBLY/{2}/{1} -name "*_genomic.fna.gz" |
            grep -v "_from_" |
            xargs gzip -dcf |
            mash sketch -k 21 -s {{ mh_sketch }} -p 2 - -I "{1}" -o "{2}/{1}"
    '

log_info Done.

exit 0
