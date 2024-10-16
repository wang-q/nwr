{%- include "header" -%}
{# Keep a blank line #}
#----------------------------#
# Usage
#----------------------------#
USAGE="
Usage: $0 [STR_IN_FLD] ...

Default values:
    STR_IN_FLD  ''

$ bash rsync.sh Klebsiella Stutzerimonas

"

if [ "$1" == "-h" ] || [ "$1" == "--help" ]; then
    echo $USAGE
    exit 0
fi

#----------------------------#
# Run
#----------------------------#
log_warn rsync.sh

touch check.lst

cat url.tsv |
    tsv-join -f check.lst -k 1 -e |
    if [ "$#" -gt 0 ]; then
        # Initialize an string to store the cmd
        result="tsv-filter --or"

        # Iterate over each argument and prepend the fixed string
        for arg in "$@"; do
            result+=" --str-in-fld '3:$arg'"
        done

        # Remove the trailing space from the result string
        result=${result% }

        # Execute the result string as a Bash command
        eval "$result"
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
            --exclude="*_genomic.gbff.gz" \
            --exclude="*_genomic.gtf.gz" \
            --exclude="*_protein.gpff.gz" \
            --exclude="*_translated_cds.faa.gz" \
            --exclude="*_wgsmaster.gbff.gz" \
            --exclude="uncompressed_checksums.txt"
    '

log_info Done.

exit 0
