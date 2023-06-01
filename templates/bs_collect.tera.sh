{%- include "header" -%}
{# Keep a blank line #}
#----------------------------#
# Usage
#----------------------------#
USAGE="
Usage: $0 [COUNT_ATTR]

Default values:
    COUNT_ATTR  50

$ bash collect.sh 100

"

if ! [ -z "$1" ]; then
    if ! [[ $1 =~ ^[0-9]+$ ]]; then
        echo >&2 "$USAGE"
        exit 1
    fi
fi

COUNT_ATTR=${1:-50}

#----------------------------#
# Run
#----------------------------#
log_warn collect.sh

log_info attributes.lst
cat sample.tsv |
    parallel --colsep '\t' --no-run-if-empty --linebuffer -k -j 4 '
        if [ -s "{3}/{1}.txt" ]; then
            cat "{3}/{1}.txt" |
                perl -nl -e '\''
                    print $1 if m(\s+\/([\w_ ]+)=);
                '\''
        fi
    ' |
    tsv-uniq --at-least ${COUNT_ATTR} | # ignore rare attributes
    grep -v "^INSDC" |
    grep -v "^ENA" \
    > attributes.lst

log_info biosample.tsv

# Headers
cat attributes.lst |
    (echo -e "#name\nBioSample" && cat) |
    tr '\n' '\t' |
    sed 's/\t$/\n/' \
    > biosample.tsv

cat sample.tsv |
    parallel --colsep '\t' --no-run-if-empty --linebuffer -k -j 1 '
        log_debug "{1}"

        cat "{3}/{1}.txt"  |
            perl -nl -MPath::Tiny -e '\''
                BEGIN {
                    our @keys = grep {/\S/} path(q{attributes.lst})->lines({chomp => 1});
                    our %stat = ();
                }

                m(\s+\/([\w_ ]+)=\"(.+)\") or next;
                my $k = $1;
                my $v = $2;
                if ( $v =~ m(\bNA|missing|Not applicable|not collected|not available|not provided|N\/A|not known|unknown\b)i ) {
                    $stat{$k} = q();
                } else {
                    $stat{$k} = $v;
                }

                END {
                    my @c;
                    for my $key ( @keys ) {
                        if (exists $stat{$key}) {
                            push @c, $stat{$key};
                        }
                        else {
                            push @c, q();
                        }
                    }
                    print join(qq(\t), q({2}), q({1}), @c);
                }
            '\''
    ' \
    >> biosample.tsv

log_info Done.

exit 0
