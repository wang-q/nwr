{%- include "header" -%}
{# Keep a blank line #}
#----------------------------#
# Run
#----------------------------#
log_warn species.sh

log_info ANI distance within species

cat species.tsv |
{% for i in ins -%}
    tsv-join -f ../{{ i }} -k 1 |
{% endfor -%}
{% for i in not_ins -%}
    tsv-join -e -f ../{{ i }} -k 1 |
{% endfor -%}
    tsv-select -f 2 |
    tsv-uniq \
    > species.lst

cat species.lst |
while read SPECIES; do
    log_debug "${SPECIES}"
    mkdir -p "${SPECIES}"

    cat species.tsv |
{% for i in ins -%}
    tsv-join -f ../{{ i }} -k 1 |
{% endfor -%}
{% for i in not_ins -%}
    tsv-join -e -f ../{{ i }} -k 1 |
{% endfor -%}
        tsv-filter --str-eq "2:${SPECIES}" |
        tsv-select -f 1 \
        > "${SPECIES}/assembly.lst"

#    echo >&2 "Number of assemblies >= 2"
    N_ASM=$(
        cat "${SPECIES}/assembly.lst" | wc -l
    )
    if [[ $N_ASM -lt 2 ]]; then
        continue
    fi

    echo >&2 "    mash distances"
    cat "${SPECIES}/assembly.lst" |
        parallel --no-run-if-empty --linebuffer -k -j 1 "
            if [[ -e ${SPECIES}/msh/{}.msh ]]; then
                echo ${SPECIES}/msh/{}.msh
            fi
        " \
        > "${SPECIES}/msh.lst"

    if [[ ! -s "${SPECIES}/mash.dist.tsv" ]]; then
        mash triangle -E -p 8 -l "${SPECIES}/msh.lst" \
            > "${SPECIES}/mash.dist.tsv"
    fi
done

log_info Done.

exit 0
