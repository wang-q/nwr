{%- include "header" -%}
{# Keep a blank line #}
#----------------------------#
# Run
#----------------------------#
log_warn check.sh

touch check.lst

# Keep only the results in the list
cat check.lst |
    sort |
    tva uniq |
    tva join -f url.tsv -k 1 \
    > tmp.list
mv tmp.list check.lst

cat url.tsv |
    tva join -f check.lst -k 1 -e |
    parallel --colsep '\t' --no-run-if-empty --linebuffer -k -j {{ parallel }} '
        if [[ ! -e "{3}/{1}" ]]; then
            exit
        fi
        log_debug "{3}\t{1}"
        cd "{3}/{1}"
        md5sum --check <(
            cat md5checksums.txt |
                grep -E "(_assembly_report\.txt|_feature_count\.txt\.gz|_genomic\.fna\.gz|_genomic\.gff\.gz|_protein\.faa\.gz)$"
            ) --status
        if [ "$?" -eq "0" ]; then
            echo "{1}" >> ../../check.lst
        else
            log_warn "{1} checksum failed"
        fi
    '

log_info Done.

exit 0
