{%- include "header" -%}
{# Keep a blank line #}
#----------------------------#
# Run
#----------------------------#
log_warn dist.sh

log_info Distances between assembly sketches
cat species.tsv |
{% for i in ins -%}
    tsv-join -f ../{{ i }} -k 1 |
{% endfor -%}
{% for i in not_ins -%}
    tsv-join -e -f ../{{ i }} -k 1 |
{% endfor -%}
    parallel --colsep '\t' --no-run-if-empty --linebuffer -k -j 1 '
        if [[ -e "{2}/msh/{1}.msh" ]]; then
            echo "{2}/msh/{1}.msh"
        fi
    ' \
    > msh.lst

mash triangle -E -p {{ parallel }} -l msh.lst \
    > mash.dist.tsv

log_info Fill distance matrix with lower triangle
tsv-select -f 1-3 mash.dist.tsv |
    (tsv-select -f 2,1,3 mash.dist.tsv && cat) |
    (
        cut -f 1 mash.dist.tsv |
            tsv-uniq |
            parallel -j 1 --keep-order 'echo -e "{}\t{}\t0"' &&
        cat
    ) \
    > mash.dist_full.tsv

log_info "Clustering via R hclust(), and grouping by cutree(h={{ mh_height }})"
cat mash.dist_full.tsv |
    Rscript -e '
        library(readr);
        library(tidyr);
        library(ape);

        pair_dist <- read_tsv(file("stdin"), col_names=F);
        tmp <- pair_dist %>%
            pivot_wider( names_from = X2, values_from = X3, values_fill = list(X3 = 1.0) )
        tmp <- as.matrix(tmp)
        mat <- tmp[,-1]
        rownames(mat) <- tmp[,1]

        dist_mat <- as.dist(mat)
        clusters <- hclust(dist_mat, method = "ward.D2")
        tree <- as.phylo(clusters)
        write.tree(phy=tree, file="tree.nwk")

        group <- cutree(clusters, h={{ mh_height }})
        groups <- as.data.frame(group)
        groups$ids <- rownames(groups)
        rownames(groups) <- NULL
        groups <- groups[order(groups$group), ]
        write_tsv(groups, "groups.tsv")
    '

log_info Done.

exit 0
