{%- include "header" -%}
{# Keep a blank line #}
#----------------------------#
# Run
#----------------------------#
log_warn dist.sh

log_info Distances between assembly sketches
cat species.tsv |
{% for i in ins -%}
    tva join -f ../{{ i }} -k 1 |
{% endfor -%}
{% for i in not_ins -%}
    tva join -e -f ../{{ i }} -k 1 |
{% endfor -%}
    parallel --colsep '\t' --no-run-if-empty --linebuffer -k -j 1 '
        if [[ -e "{2}/msh/{1}.msh" ]]; then
            echo "{2}/msh/{1}.msh"
        fi
    ' \
    > msh.lst

mash triangle -E -p {{ parallel }} -l msh.lst \
    > mash.dist.tsv

log_info Pairwise distances to phylip matrix
necom mat to-phylip mash.dist.tsv -o mash.dist.phylip

log_info "Clustering via necom clust hier --method ward"
necom clust hier --method ward mash.dist.phylip -o tree.nwk

log_info "Grouping by necom clust cut --height {{ mh_height }}"
necom clust cut --height {{ mh_height }} tree.nwk -o groups.tsv

log_info Done.

exit 0
