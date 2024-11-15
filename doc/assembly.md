# Download files from NCBI Assembly

## Strain info

```shell
cat ~/.nwr/assembly_summary_refseq.txt |
    sed '1d' |
    tsv-summarize -H --missing-count infraspecific_name,isolate,biosample |
    rgr md stdin --fmt

# infraspecific_name
cat ~/.nwr/assembly_summary_refseq.txt |
    sed '1d' |
    tsv-select -H -f infraspecific_name |
    perl -nla -F"=" -e 'print $F[0]' |
    keep-header -- sort |
    uniq -c
#      1 infraspecific_name
#     40 breed
#     92 cultivar
#     77 ecotype
#  53829 na
# 356287 strain

cat ~/.nwr/assembly_summary_genbank.txt |
    sed '1d' |
    tsv-select -H -f infraspecific_name |
    perl -nla -F"=" -e 'print $F[0]' |
    keep-header -- sort |
    uniq -c
#      1 infraspecific_name
#    453 breed
#   1886 cultivar
#    802 ecotype
# 876578 na
#1759454 strain

cat ~/.nwr/assembly_summary_refseq.txt ~/.nwr/assembly_summary_genbank.txt |
    grep -v "^#" |
    tsv-select -f 9 | # infraspecific_name
    perl -nla -F"=" -e 'print $F[1]' |
    sort |
    uniq -c |
    sort -nr |
    head
# 930439
#   2482 Human
#   2451 clinical isolate of L. monocytogenes
#   2362 MSSA
#   1363 MRSA
#    927 microbial
#    814 Escherichia coli
#    788 CT18
#    781 n/a
#    710 ExPEC

# String length
cat ~/.nwr/assembly_summary_refseq.txt |
    sed '1d' |
    tsv-select -H -f organism_name,infraspecific_name,asm_name,ftp_path |
    sed '1d' |
    perl -nla -F"\t" -e 'print join qq(\t), map {length} @F ;' |
    tsv-summarize --exclude-missing --max 1,2,3,4
#91      88      92      165

```

| infraspecific_name_missing_count | isolate_missing_count | biosample_missing_count |
|---------------------------------:|----------------------:|------------------------:|
|                                0 |                     0 |                       0 |

## Reference genomes

```shell
cd ~/Scripts/nwr/doc/

nwr member Bacteria Archaea -r family |
    grep -v -i "Candidatus " |
    grep -v -i "candidate " |
    grep -v " sp." |
    grep -v " spp." |
    sed '1d' |
    sort -n -k1,1 \
    > family.list.tsv

wc -l family.list.tsv
#707 family.list.tsv

FAMILY=$(
    cat family.list.tsv |
        cut -f 1 |
        tr "\n" "," |
        sed 's/,$//'
)

echo "
.headers ON
    SELECT
        *
    FROM ar
    WHERE 1=1
        AND family_id IN ($FAMILY)
        AND refseq_category IN ('reference genome')
    " |
    sqlite3 -tabs ~/.nwr/ar_refseq.sqlite \
    > reference.tsv

cat reference.tsv |
    tsv-select -H -f organism_name,species,genus,ftp_path,assembly_level \
    > raw.tsv

cat raw.tsv |
    grep -v '^#' |
    rgr dedup stdin |
    perl ~/Scripts/withncbi/taxon/abbr_name.pl -c "1,2,3" -s '\t' -m 3 --shortsub |
    (echo -e '#name\tftp_path\torganism\tassembly_level' && cat ) |
    perl -nl -a -F"," -e '
        BEGIN{my %seen};
        /^#/ and print and next;
        /^organism_name/i and next;
        $seen{$F[3]}++; # ftp_path
        $seen{$F[3]} > 1 and next;
        $seen{$F[5]}++; # abbr_name
        $seen{$F[5]} > 1 and next;
        printf qq{%s\t%s\t%s\t%s\n}, $F[5], $F[3], $F[1], $F[4];
        ' |
    keep-header -- sort -k3,3 -k1,1 \
    > Bacteria.assembly.tsv

```

## File format: .assembly.tsv

A TAB-delimited file for downloading assembly files.

| Col |  Type  | Description                                              |
|----:|:------:|:---------------------------------------------------------|
|   1 | string | #name: species + infraspecific_name + assembly_accession |
|   2 | string | ftp_path                                                 |
|   3 | string | biosample                                                |
|   4 | string | species                                                  |
|   5 | string | assembly_level                                           |
