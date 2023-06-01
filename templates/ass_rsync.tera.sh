{%- include "header" -%}
{# Keep a blank line #}
#----------------------------#
# Run
#----------------------------#
log_warn rsync.sh

touch check.lst

cat url.tsv |
    tsv-join -f check.lst -k 1 -e |
    parallel --colsep '\t' --no-run-if-empty --linebuffer -k -j 4 '
        echo >&2
        log_info "{3}\t{1}"
        mkdir -p "{3}/{1}"
        rsync -avP --no-links {2}/ {3}/{1}/ --exclude="assembly_status.txt"
    '

log_info Done.

exit 0
