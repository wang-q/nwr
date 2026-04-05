{%- include "header" -%}
{# Keep a blank line #}
#----------------------------#
# Run
#----------------------------#
log_warn rank.sh

{% for rank in count_ranks -%}
log_info "Count {{ rank }}"

log_debug "{{ rank }}.lst"
cat strains.taxon.tsv |
    tva select -f {{ rank_col_of[rank] }} |
    tva uniq |
    grep -v "NA" |
    sort \
    > {{ rank }}.lst

log_debug "{{ rank }}.count.tsv"
cat {{ rank }}.lst |
    parallel --no-run-if-empty --linebuffer -k -j {{ parallel }} '
        n_species=$(
            cat strains.taxon.tsv |
                tva filter --str-eq "{{ rank_col_of[rank] }}:{}" |
                tva select -f {{ rank_col_of[rank] }},2 |
                tva uniq |
                wc -l
        )

        n_strains=$(
            cat strains.taxon.tsv |
                tva filter --str-eq "{{ rank_col_of[rank] }}:{}" |
                tva select -f {{ rank_col_of[rank] }},1 |
                tva uniq |
                wc -l
        )

        printf "%s\t%d\t%d\n" {} ${n_species} ${n_strains}
    ' |
    tva sort -k 1 |
    (echo -e '{{ rank }}\t#species\t#strains' && cat) \
    > {{ rank }}.count.tsv

{% endfor -%}
{# Keep a blank line #}
log_info Done.

exit 0
