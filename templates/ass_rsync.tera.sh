{%- include "header" -%}
{# Keep a blank line #}
#----------------------------#
# Usage
#----------------------------#
USAGE="
Usage: $0 [STR_IN_FLD]

Default values:
    STR_IN_FLD  ''

$ bash rsync.sh Klebsiella

"

STR_IN_FLD=${1:-}

#----------------------------#
# Run
#----------------------------#
log_warn rsync.sh

touch check.lst

cat url.tsv |
    tsv-join -f check.lst -k 1 -e |
    if ! [ -z "$1" ]; then
        tsv-filter --str-in-fld "3:${STR_IN_FLD}"
    else
        tsv-uniq
    fi |
    parallel --colsep '\t' --no-run-if-empty --linebuffer -k -j 4 '
        echo >&2
        log_info "{3}\t{1}"
        mkdir -p "{3}/{1}"
        rsync -avP --no-links {2}/ {3}/{1}/ \
            --exclude="*assembly_structure/*" \
            --exclude="annotation_hashes.txt" \
            --exclude="*_ani_contam_ranges.tsv" \
            --exclude="*_ani_report.txt" \
            --exclude="assembly_status.txt" \
            --exclude="*_assembly_stats.txt" \
            --exclude="*_fcs_report.txt" \
            --exclude="*_feature_table.txt.gz" \
            --exclude="*_genomic_gaps.txt.gz" \
            --exclude="*_genomic.gtf.gz" \
            --exclude="*_protein.gpff.gz" \
            --exclude="*_translated_cds.faa.gz" \
            --exclude="*_wgsmaster.gbff.gz" \
            --exclude="uncompressed_checksums.txt"
    '

log_info Done.

exit 0
