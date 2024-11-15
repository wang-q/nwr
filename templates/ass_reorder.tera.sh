{%- include "header" -%}
{# Keep a blank line #}
#----------------------------#
# Run
#----------------------------#
log_warn reorder.sh

log_info "Put the misplaced directory in the right place"
find . -maxdepth 3 -mindepth 2 -type f -name "*_genomic.fna.gz" |
    grep -v "_from_" |
    parallel --no-run-if-empty --linebuffer -k -j 1 '
        echo {//}
    ' |
    tr "/" "\t" |
    perl -nla -F"\t" -MPath::Tiny -e '
        BEGIN {
            our %species_of = map {(split)[0, 2]}
                grep {/\S/}
                path(q{url.tsv})->lines({chomp => 1});
        }

        # Should like ".       Saccharomyces_cerevisiae        Sa_cer_S288C"
        @F != 3 and print and next;

        # Assembly is not in the list
        if (! exists $species_of{$F[2]} ) {
            print;
            next;
        }

        # species is the correct one
        if ($species_of{$F[2]} ne $F[1]) {
            print;
            next;
        }
    ' |
    perl -nla -F"\t" -e '
        m((GC[FA]_\d+_\d+$)) or next;
        my $acc = $1;
        my $dir = join q(/), @F;
        print join qq(\t), $dir, $acc;
    ' \
    > misplaced.tsv

cat misplaced.tsv |
    parallel --colsep '\t' --no-run-if-empty --linebuffer -k -j 4 '
        SPECIES=$(
            tsv-filter url.tsv --str-in-fld "1:{2}" |
                tsv-select -f 3
            )
        NAME=$(
            tsv-filter url.tsv --str-in-fld "1:{2}" |
                tsv-select -f 1
            )
        if [ ! -z "${NAME}" ]; then
            if [ -e "${SPECIES}/${NAME}" ]; then
                echo >&2 "${SPECIES}/${NAME} exists"
            else
                echo >&2 "Moving {1} to ${SPECIES}/${NAME}"
                mkdir -p "${SPECIES}"
                mv {1} "${SPECIES}/${NAME}"
            fi
        fi
    '

log_info "Temporary files, possibly caused by an interrupted rsync process"
find . -type f -name ".*" |
    grep -v "DS_Store" \
    > remove.lst

log_info "List dirs (species/assembly) not in the list"
cat url.tsv |
    tsv-select -f 3 |
    rgr dedup stdin |
while read SPECIES; do
    find "./${SPECIES}" -maxdepth 1 -mindepth 1 -type d |
        tr "/" "\t" |
        tsv-select -f 3 |
        tsv-join --exclude -k 1 -f url.tsv -d 1 |
        xargs -I[] echo "./${SPECIES}/[]"
done \
    >> remove.lst

log_info "List dirs (species) not in the list"
find . -maxdepth 1 -mindepth 1 -type d |
    tr "/" "\t" |
    tsv-select -f 2 |
    tsv-join --exclude -k 3 -f url.tsv -d 1 |
    xargs -I[] echo "./[]" \
    >> remove.lst

if [ -s remove.lst ]; then
    log_info "remove.lst is not empty."
fi

log_info Done.

exit 0
