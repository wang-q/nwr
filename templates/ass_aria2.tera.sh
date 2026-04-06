{%- include "header" -%}
{# Keep a blank line #}
#----------------------------#
# Usage
#----------------------------#
USAGE="
Usage: $0 [STR_IN_FLD] ...

Default values:
    STR_IN_FLD  ''

$ bash aria2.sh Klebsiella Stutzerimonas

"

if [ "$1" == "-h" ] || [ "$1" == "--help" ]; then
    echo $USAGE
    exit 0
fi

#----------------------------#
# Run
#----------------------------#
log_warn aria2.sh

touch check.lst

cat url.tsv |
    tva join -f check.lst -k 1 -e |
    if [ "$#" -gt 0 ]; then
        result="tva filter --or"
        for arg in "$@"; do
            result+=" --str-in-fld '3:$arg'"
        done
        result=${result% }
        eval "$result"
    else
        tva uniq
    fi |
    parallel --colsep '\t' --no-run-if-empty --linebuffer -k -j 4 '
        echo >&2
        log_info "{3}\t{1}"
        mkdir -p "{3}/{1}"

        # Download md5checksums.txt
        curl -sL "{2}/md5checksums.txt" -o "{3}/{1}/md5checksums.txt"

        # Extract file list from md5checksums.txt
        cat "{3}/{1}/md5checksums.txt" |
            grep -v "assembly_structure" |
            grep -v "Evidence_alignments" |
            grep -v "Gnomon_models" |
            grep -v "RNASeq_coverage_graphs" |
            grep -v "RefSeq_transcripts_alignments" |
            grep -v "annotation_hashes.txt" |
            grep -v "_ani_contam_ranges.tsv" |
            grep -v "_ani_report.txt" |
            grep -v "assembly_status.txt" |
            grep -v "_assembly_stats.txt" |
            grep -v "_fcs_report.txt" |
            grep -v "_feature_table.txt.gz" |
            grep -v "_genomic_gaps.txt.gz" |
            grep -v "_genomic.gbff.gz" |
            grep -v "_genomic.gtf.gz" |
            grep -v "_protein.gpff.gz" |
            grep -v "_translated_cds.faa.gz" |
            grep -v "_wgsmaster.gbff.gz" |
            grep -v "uncompressed_checksums.txt" |
            perl -nl -e '"'"'m/^\S+\s+\.\/(\S+)$/ and print $1'"'"' > "{3}/{1}/download.txt"

        # Generate aria2 input file
        cat "{3}/{1}/download.txt" |
            perl -nl -e '"'"'print qq({2}/$_)'"'"' > "{3}/{1}/aria2.txt"

        # Download files with aria2
        aria2c -x 16 -s 16 -j 4 -c -i "{3}/{1}/aria2.txt" -d "{3}/{1}"
    '

log_info Done.

exit 0
