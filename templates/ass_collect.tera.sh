{%- include "header" -%}
{# Keep a blank line #}
#----------------------------#
# Run
#----------------------------#
log_warn collect.sh

touch check.lst

echo -e "name\t{{ ass_columns | join(sep="\t") }}" \
    > collect.tsv

cat url.tsv |
    tsv-join -f check.lst -k 1 |
    parallel --colsep '\t' --no-run-if-empty --linebuffer -k -j 1 '
        log_debug "{3}\t{1}"
        find "{3}/{1}" -type f -name "*_assembly_report.txt" |
            xargs cat |
            perl -nl -e '\''
                BEGIN { our %stat = (); }

                m{^#\s+} or next;
                s/^#\s+//;
                @O = split /\:\s*/;
                scalar @O == 2 or next;
                $O[0] =~ s/\s*$//g;
                $O[0] =~ s/\W/_/g;
                $O[1] =~ /([\w =.-]+)/ or next;
                $stat{$O[0]} = $1;

                END {
                    my @c;
                    for my $key ( qw( {{ ass_columns | join(sep=" ") }} ) ) {
                        if (exists $stat{$key}) {
                            push @c, $stat{$key};
                        }
                        else {
                            push @c, q();
                        }
                    }
                    print join(qq(\t), q({1}), @c);
                }
            '\'' \
            >> collect.tsv
    '

log_info Done.

exit 0
