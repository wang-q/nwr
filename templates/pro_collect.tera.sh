{%- include "header" -%}
{# Keep a blank line #}
#----------------------------#
# Run
#----------------------------#
log_warn Protein/collect.sh

log_info "Protein/all.pro.fa.gz"
cat species.tsv |
{% for i in ins -%}
    tsv-join -f ../{{ i }} -k 1 |
{% endfor -%}
{% for i in not_ins -%}
    tsv-join -e -f ../{{ i }} -k 1 |
{% endfor -%}
    parallel --colsep '\t' --no-run-if-empty --linebuffer -k -j 1 '
        if [[ ! -d "../ASSEMBLY/{2}/{1}" ]]; then
            exit
        fi

        gzip -dcf ../ASSEMBLY/{2}/{1}/*_protein.faa.gz
    ' |
     pigz -p4 \
     > all.pro.fa.gz

log_info "Protein/all.uniq.fa.gz"
gzip -dcf all.pro.fa.gz |
    perl -nl -e '
        BEGIN { our %seen; our $h; }

        if (/^>/) {
            $h = (split(" ", $_))[0];
            $seen{$h}++;
            $_ = $h;
        }
        print if $seen{$h} == 1;
    ' |
    pigz -p4 \
    > all.uniq.fa.gz

log_info "Protein/all.replace.fa.gz"
rm -f all.strain.tsv all.replace.fa.gz

cat species.tsv |
{% for i in ins -%}
    tsv-join -f ../{{ i }} -k 1 |
{% endfor -%}
{% for i in not_ins -%}
    tsv-join -e -f ../{{ i }} -k 1 |
{% endfor -%}
    parallel --colsep '\t' --no-run-if-empty --linebuffer -k -j 1 '
        if [[ ! -d "../ASSEMBLY/{2}/{1}" ]]; then
            exit
        fi

        gzip -dcf ../ASSEMBLY/{2}/{1}/*_protein.faa.gz |
            grep "^>" |
            cut -d" " -f 1 |
            sed "s/^>//" |
            perl -nl -e '\''
                $n = $_;
                $s = $n;
                $s =~ s/\.\d+//;
                printf qq(%s\t%s_%s\t%s\n), $n, {1}, $s, {1};
            '\'' \
            > {1}.replace.tsv

        cut -f 2,3 {1}.replace.tsv >> all.strain.tsv

        faops replace -s \
            ../ASSEMBLY/{2}/{1}/*_protein.faa.gz \
            <(cut -f 1,2 {1}.replace.tsv) \
            stdout |
            pigz -p4 \
            >> all.replace.fa.gz

        rm {1}.replace.tsv
    '

log_info Done.

exit 0
