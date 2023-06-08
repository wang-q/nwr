{%- include "header" -%}
{# Keep a blank line #}
#----------------------------#
# Usage
#----------------------------#
USAGE="
Usage: $0 [N_COUNT]

Default values:
    N_COUNT greater than or equal to 1

$ bash Count/lineage.sh 5

"

if ! [ -z "$1" ]; then
    if ! [[ $1 =~ ^[0-9]+$ ]]; then
        echo >&2 "$USAGE"
        exit 1
    fi
fi

N_COUNT=${1:-1}

#----------------------------#
# Run
#----------------------------#
log_warn Count/lineage.sh

{% set cols = count_lineages | length() + 1 -%}

cat strains.taxon.tsv |
    tsv-summarize -g {% for rank in count_lineages %}{{ rank_col_of[rank] ~ "," }}{% endfor %}2 --count |
    tsv-filter --ge "{{ count_lineages | length() + 2 }}:${N_COUNT}" |
    tsv-sort -k{{ count_lineages | length() + 1 }},{{ count_lineages | length() + 1 }} |
{% for i in range(start=1, end=cols) | reverse -%}
    tsv-sort -k{{ i }},{{ i }} |
{% endfor -%}
    perl -nla -F'\t' -e '
            BEGIN {
{% for rank in count_lineages -%}
                our ${{ rank }} = q();
{% endfor -%}
            }
{# Keep a blank line #}
{% for rank in count_lineages -%}
            # record the current {{ rank }}
            if ($F[ {{ loop.index - 1 }} ] eq ${{ rank }}) {
                printf qq(\t);
            } else {
                ${{ rank }} = $F[ {{ loop.index - 1 }} ];
                printf qq(${{ rank }}\t);
            }
{% endfor -%}
{# Keep a blank line #}
            print join qq(\t), (
                $F[ {{ count_lineages | length() + 0 }} ],
                $F[ {{ count_lineages | length() + 1 }} ],
            );
        ' |
    (echo -e '#{% for rank in count_lineages %}{{ rank ~ "\t" }}{% endfor %}species\tcount' && cat) \
    > lineage.count.tsv

{# Keep a blank line #}
log_info Done.

exit 0
