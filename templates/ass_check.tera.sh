{%- include "header" -%}
{# Keep a blank line #}
#----------------------------#
# Run
#----------------------------#
log_warn check.sh

touch check.lst

# Keep only the results in the list
cat check.lst |
    tsv-uniq |
    tsv-join -f url.tsv -k 1 \
    > tmp.list
mv tmp.list check.lst

cat url.tsv |
    tsv-join -f check.lst -k 1 -e |
    parallel --colsep '\t' --no-run-if-empty --linebuffer -k -j 4 '
        if [[ ! -e "{3}/{1}" ]]; then
            exit
        fi
        log_debug "{3}\t{1}"
        cd "{3}/{1}"
        md5sum --check md5checksums.txt --status
        if [ "$?" -eq "0" ]; then
            echo "{1}" >> ../../check.lst
        else
            log_warn "{1} checksum failed"
        fi
    '

log_info Done.

exit 0
