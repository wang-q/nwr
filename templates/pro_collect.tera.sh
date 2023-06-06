{%- include "header" -%}
{# Keep a blank line #}
#----------------------------#
# Run
#----------------------------#
log_warn Protein/collect.sh

#----------------------------#
# all.pro.fa
#----------------------------#
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

#----------------------------#
# all.replace.fa
#----------------------------#
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

log_info "Protein/all.size.tsv"
(echo -e "#name\tstrain" && cat all.strain.tsv)  \
    > temp &&
    mv temp all.strain.tsv

faops size all.replace.fa.gz > all.replace.sizes

(echo -e "#name\tsize" && cat all.replace.sizes) > all.size.tsv

rm all.replace.sizes

#----------------------------#
# `all.info.tsv`
#----------------------------#
log_info "Protein/all.annotation.tsv"
cat species.tsv |
{% for i in ins -%}
    tsv-join -f ../{{ i }} -k 1 |
{% endfor -%}
{% for i in not_ins -%}
    tsv-join -e -f ../{{ i }} -k 1 |
{% endfor -%}
    parallel --colsep '\t' --no-run-if-empty --linebuffer -k -j 4 '
        if [[ ! -d "../ASSEMBLY/{2}/{1}" ]]; then
            exit
        fi

        gzip -dcf ../ASSEMBLY/{2}/{1}/*_protein.faa.gz |
            grep "^>" |
            sed "s/^>//" |
            perl -nl -e '\'' /\[.+\[/ and s/\[/\(/; print; '\'' |
            perl -nl -e '\'' /\].+\]/ and s/\]/\)/; print; '\'' |
            perl -nl -e '\'' s/\s+\[.+?\]$//g; print; '\'' |
            sed "s/MULTISPECIES: //g" |
            perl -nl -e '\''
                /^(\w+)\.\d+\s+(.+)$/ or next;
                printf qq(%s_%s\t%s\n), {1}, $1, $2;
            '\''
    ' \
    > all.annotation.tsv

(echo -e "#name\tannotation" && cat all.annotation.tsv) \
    > temp &&
    mv temp all.annotation.tsv

log_info "Protein/all.info.tsv"
tsv-join \
    all.strain.tsv \
    --data-fields 1 \
    -f all.size.tsv \
    --key-fields 1 \
    --append-fields 2 \
    > all.strain_size.tsv

tsv-join \
    all.strain_size.tsv \
    --data-fields 1 \
    -f all.annotation.tsv \
    --key-fields 1 \
    --append-fields 2 \
    > all.info.tsv


#----------------------------#
# Counts
#----------------------------#
log_info "Counts"

printf "#item\tcount\n" \
    > counts.tsv

gzip -dcf all.pro.fa.gz |
    grep "^>" |
    wc -l |
    perl -nl -MNumber::Format -e '
        printf qq(Proteins\t%s\n), Number::Format::format_number($_, 0,);
        ' \
    >> counts.tsv

gzip -dcf all.pro.fa.gz |
    grep "^>" |
    tsv-uniq |
    wc -l |
    perl -nl -MNumber::Format -e '
        printf qq(Unique headers and annotations\t%s\n), Number::Format::format_number($_, 0,);
        ' \
    >> counts.tsv

gzip -dcf all.uniq.fa.gz |
    grep "^>" |
    wc -l |
    perl -nl -MNumber::Format -e '
        printf qq(Unique proteins\t%s\n), Number::Format::format_number($_, 0,);
        ' \
    >> counts.tsv

gzip -dcf all.replace.fa.gz |
    grep "^>" |
    wc -l |
    perl -nl -MNumber::Format -e '
        printf qq(all.replace.fa\t%s\n), Number::Format::format_number($_, 0,);
        ' \
    >> counts.tsv

cat all.annotation.tsv |
    wc -l |
    perl -nl -MNumber::Format -e '
        printf qq(all.annotation.tsv\t%s\n), Number::Format::format_number($_, 0,);
        ' \
    >> counts.tsv

cat all.info.tsv |
    wc -l |
    perl -nl -MNumber::Format -e '
        printf qq(all.info.tsv\t%s\n), Number::Format::format_number($_, 0,);
        ' \
    >> counts.tsv

log_info Done.

exit 0
