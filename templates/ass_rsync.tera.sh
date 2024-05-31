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
        rsync -avP --no-links {2}/ {3}/{1}/ \
            --exclude="*assembly_structure/*" \
            --exclude="assembly_status.txt" \
            --exclude="annotation_hashes.txt" \
            --exclude="*_feature_table.txt.gz" \
            --exclude="*_genomic_gaps.txt.gz" \
            --exclude="*_protein.gbff.gz" \
            --exclude="*_wgsmaster.gbff.gz"
    '

log_info Done.

exit 0
