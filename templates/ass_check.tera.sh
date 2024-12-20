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
    rgr dedup stdin |
    tsv-join -f url.tsv -k 1 \
    > tmp.list
mv tmp.list check.lst

cat url.tsv |
    tsv-join -f check.lst -k 1 -e |
    parallel --colsep '\t' --no-run-if-empty --linebuffer -k -j {{ parallel }} '
        if [[ ! -e "{3}/{1}" ]]; then
            exit
        fi
        log_debug "{3}\t{1}"
        cd "{3}/{1}"
        md5sum --check <(
            cat md5checksums.txt |
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
                grep -v "uncompressed_checksums.txt"
            ) --status
        if [ "$?" -eq "0" ]; then
            echo "{1}" >> ../../check.lst
        else
            log_warn "{1} checksum failed"
        fi
    '

log_info Done.

exit 0
