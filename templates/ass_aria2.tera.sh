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

# C_myc_ATHUM6906_GCA_036429605_1	https://ftp.ncbi.nlm.nih.gov/genomes/all/GCA/036/429/605/GCA_036429605.1_ASM3642960v1	Cladobotryum_mycophilum
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
    parallel --colsep '\t' --rpl '{} uq()' --no-run-if-empty --linebuffer -k -j 4 '
        # GNU parallel --colsep automatically quotes fields containing special 
        #   chars (spaces, slashes, quotes, etc.) to prevent shell misinterpretation.
        # --rpl disables quoting for all replacement strings

        echo >&2
        log_info "{3}\t{1}"
        mkdir -p "{3}/{1}"

        # Download md5checksums.txt if not exists or empty
        MD5_URL="{2}/md5checksums.txt"
        MD5_FILE="{3}/{1}/md5checksums.txt"
        if [[ ! -e "$MD5_FILE" ]] || [[ ! -s "$MD5_FILE" ]]; then
            log_info "curl -sL --show-error $MD5_URL -o $MD5_FILE"
            #  curl -sL --show-error 'https://ftp.ncbi.nlm.nih.gov/genomes/all/GCA/007/896/495/GCA_007896495.1_ASM789649v1'/md5checksums.txt -o Trichoderma_viride/T_viri_Tv_1511_GCA_007896495_1/md5checksums.txt
            if ! curl -sL --show-error "$MD5_URL" -o "$MD5_FILE"; then
                log_warn "Failed to download md5checksums.txt for {1}"
                exit
            fi
        fi

        # Extract file list from md5checksums.txt and generate aria2 input file
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
            perl -nl -e '\''
                my $fn = (split /\s+/)[-1];
                $fn =~ s/^\.\///; print qq({2}/$fn)
                '\'' > "{3}/{1}/aria2.txt"

        # Check if aria2 input file is empty
        if [ ! -s "{3}/{1}/aria2.txt" ]; then
            log_warn "No files to download for {1}"
            exit
        fi

        # Download files with aria2 (single connection per file, parallel handled by GNU parallel)
        if ! aria2c -x 1 -s 1 -j 1 -c --file-allocation=none -i "{3}/{1}/aria2.txt" -d "{3}/{1}"; then
            log_warn "Failed to download files for {1}"
            exit
        fi
    '

log_info Done.

exit 0
