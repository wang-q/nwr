{%- include "header" -%}
{# Keep a blank line #}
#----------------------------#
# Usage
#----------------------------#
USAGE="
Usage: $0 [LEN_N50] [N_CONTIG] [LEN_SUM]

Default values:
    LEN_N50     longer than 100000
    N_CONTIG    less than   1000
    LEN_SUM     longer than 1000000

$ bash n50.sh 100000 100

"

if ! [ -z "$1" ]; then
    if ! [[ $1 =~ ^[0-9]+$ ]]; then
        echo >&2 "$USAGE"
        exit 1
    fi
fi

LEN_N50=${1:-100000}
N_CONTIG=${2:-1000}
LEN_SUM=${3:-1000000}

#----------------------------#
# Run
#----------------------------#
log_warn n50.sh

touch n50.tsv

# Keep only the results in the list
cat n50.tsv |
    tsv-uniq |
    tsv-filter -H --gt 2:0 | # unfinished downloads
    tsv-join -f url.tsv -k 1 \
    > tmp.tsv
mv tmp.tsv n50.tsv

# Calculate N50 not in the list
cat url.tsv |
    tsv-join -f n50.tsv -k 1 -e |
    parallel --colsep '\t' --no-run-if-empty --linebuffer -k -j 4 '
        if [[ ! -e "{3}/{1}" ]]; then
            exit
        fi
        log_debug "{3}\t{1}"

        find "{3}/{1}" -type f -name "*_genomic.fna.gz" |
            grep -v "_from_" | # exclude CDS and rna
            xargs cat |
            faops n50 -C -S stdin |
            (echo -e "name\t{1}" && cat) |
            datamash transpose
    ' \
    > tmp1.tsv

# Combine new results with the old ones
cat tmp1.tsv n50.tsv |
    tsv-uniq | # keep the first header
    keep-header -- sort \
    > tmp2.tsv
mv tmp2.tsv n50.tsv
rm tmp*.tsv

# Filter results with custom criteria
cat n50.tsv |
    tsv-filter -H --ge "N50:${LEN_N50}" |
    tsv-filter -H --le "C:${N_CONTIG}" |
    tsv-filter -H --ge "S:${LEN_SUM}" |
    tr "\t" "," \
    > n50.pass.csv

log_info Done.

exit 0
