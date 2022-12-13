# Download files from NCBI Assembly

## Strain info

```shell
cat ~/.nwr/assembly_summary_refseq.txt |
    sed '1d' |
    tsv-summarize -H --missing-count infraspecific_name --not-missing-count infraspecific_name |
    mlr --itsv --omd cat

cat ~/.nwr/assembly_summary_refseq.txt |
    sed '1d' |
    tsv-summarize -H --missing-count isolate --not-missing-count isolate |
    mlr --itsv --omd cat

```

| infraspecific_name_missing_count | infraspecific_name_not_missing_count |
|----------------------------------|--------------------------------------|
| 19179                            | 253662                               |

| isolate_missing_count | isolate_not_missing_count |
|-----------------------|---------------------------|
| 254850                | 17991                     |

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
#697 family.list.tsv

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
    tsv-uniq |
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

| Col |  Type  | Description    |
|----:|:------:|:---------------|
|   1 | string | #name          |
|   2 | string | ftp_path       |
|   3 | string | organism       |
|   4 | string | assembly_level |

