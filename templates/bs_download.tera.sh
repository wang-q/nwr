{%- include "header" -%}
{# Keep a blank line #}
#----------------------------#
# Run
#----------------------------#
log_warn download.sh

cat sample.tsv |
    parallel --colsep '\t' --no-run-if-empty --linebuffer -k -j 4 '
        mkdir -p "{3}"
        if [ ! -s "{3}/{1}.txt" ]; then
            echo >&2 -e "==> {1}\t{2}\t{3}"
            curl -fsSL "https://www.ncbi.nlm.nih.gov/biosample/?term={1}&report=full&format=text" -o "{3}/{1}.txt"
        fi
    '

log_info Done.

exit 0
