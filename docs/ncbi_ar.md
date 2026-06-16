# NCBI Assembly Reports

## Preparations

```shell
cbp install nwr
cbp install sqlite3
cbp install tva
```

Requires SQLite version 3.34 or above. `sqlite` that comes with mac does not work.

## NCBI Taxonomy Statistics

```shell
curl -L "https://www.ncbi.nlm.nih.gov/Taxonomy/taxonomyhome.html/index.cgi?chapter=statistics&?&unclassified=hide&uncultured=hide" |
    tva from html -q 'table[bgcolor="#CCCCFF"] table[bgcolor="#FFFFFF"] tr td text{}' |
    grep '\S' |
    paste -d $'\t' - - - - - - |
    tva to md --right 2-6
```

| Ranks:        | higher taxa |   genus | species | lower taxa |     total |
| ------------- | ----------: | ------: | ------: | ---------: | --------: |
| Archaea       |           0 |     340 |   1,200 |      2,290 |     2,290 |
| Bacteria      |           0 |   5,782 |  33,615 |     90,218 |    90,218 |
| Eukaryota     |           0 | 104,261 | 631,437 |    804,447 |   804,447 |
| Fungi         |           0 |   8,095 |  74,507 |     88,460 |    88,460 |
| Metazoa       |           0 |  75,546 | 340,416 |    453,240 |   453,240 |
| Viridiplantae |           0 |  16,338 | 198,532 |    237,280 |   237,280 |
| Viruses       |          36 |   3,493 |  14,612 |    200,795 |   201,328 |
| All taxa      |          54 | 113,878 | 700,762 |  1,097,758 | 1,118,224 |

## NCBI ASSEMBLY

* assembly_level

```shell
for C in refseq genbank; do
    cat ~/.nwr/assembly_summary_${C}.txt |
        sed '1d' |
        tva stats -H -g assembly_level,genome_rep --count |
        tva keep-header -- sort |
        tva to md --fmt

    echo -e "\nTable: ${C}\n\n"
done
```

| assembly_level  | genome_rep |   count |
| --------------- | ---------- | ------: |
| Chromosome      | Full       |   8,629 |
| Chromosome      | Partial    |     355 |
| Complete Genome | Full       |  76,533 |
| Complete Genome | Partial    |       7 |
| Contig          | Full       | 280,107 |
| Contig          | Partial    |      30 |
| Scaffold        | Full       | 158,032 |

Table: refseq


| assembly_level  | genome_rep |     count |
| --------------- | ---------- | --------: |
| Chromosome      | Full       |    44,020 |
| Chromosome      | Partial    |     1,196 |
| Complete Genome | Full       |   309,100 |
| Complete Genome | Partial    |       131 |
| Contig          | Full       | 2,549,556 |
| Contig          | Partial    |       933 |
| Scaffold        | Full       |   515,294 |
| Scaffold        | Partial    |       363 |

Table: genbank

## Example 1: count qualified assemblies of Eukaryote groups

```shell
ARRAY=(
    # Animals - Metazoa - kingdom
    'Flatworms::Platyhelminthes' # phylum
    'Roundworms::Nematoda'
    'Insects::Hexapoda' # subphylum
    'Reptiles::Testudines' # order
    'Reptiles::Lepidosauria'
    'Reptiles::Crocodylia'
    'Fishes::Chondrichthyes' # class
    'Fishes::Dipnoi'
    'Fishes::Actinopterygii'
    'Fishes::Hyperotreti'
    'Fishes::Hyperoartia'
    'Fishes::Coelacanthimorpha'
    'Mammals::Mammalia'
    'Birds::Aves'
    'Amphibians::Amphibia'
    # Fungi - kindom
    'Ascomycetes::Ascomycota' # phylum
    'Basidiomycetes::Basidiomycota'
    # Plants - Viridiplantae
    'Green Plants::Viridiplantae'
    'Land Plants::Embryophyta'
    # Protists
    'Apicomplexans::Apicomplexa'
    'Kinetoplasts::Kinetoplastida'
)

echo -e "GROUP_NAME\tSCI_NAME\tComplete Genome\tChromosome\tScaffold\tContig" \
    > groups.tsv

for item in "${ARRAY[@]}" ; do
    GROUP_NAME="${item%%::*}"
    SCI_NAME="${item##*::}"

    GENUS=$(
        nwr member ${SCI_NAME} -r genus |
            grep -v -i "Candidatus " |
            grep -v -i "candidate " |
            sed '1d' |
            cut -f 1 |
            tr "\n" "," |
            sed 's/,$/\)/' |
            sed 's/^/\(/'
    )

    printf "$GROUP_NAME\t$SCI_NAME\t"

    for L in 'Complete Genome' 'Chromosome' 'Scaffold' 'Contig'; do
        echo "
            SELECT
                COUNT(*)
            FROM ar
            WHERE 1=1
                AND genus_id IN $GENUS
                AND assembly_level IN ('$L')
            " |
            sqlite3 -tabs ~/.nwr/ar_refseq.sqlite
    done |
    tr "\n" "\t" |
    sed 's/\t$//'

    echo;
done \
    >> groups.tsv

cat groups.tsv |
    tva to md --num

```

| GROUP_NAME     | SCI_NAME          | Complete Genome | Chromosome | Scaffold | Contig |
| -------------- | ----------------- | --------------: | ---------: | -------: | -----: |
| Flatworms      | Platyhelminthes   |               0 |          2 |        5 |      0 |
| Roundworms     | Nematoda          |               1 |          4 |        3 |      0 |
| Insects        | Hexapoda          |               1 |        208 |      105 |     30 |
| Reptiles       | Testudines        |               0 |         17 |        1 |      1 |
| Reptiles       | Lepidosauria      |               0 |         25 |        9 |      1 |
| Reptiles       | Crocodylia        |               0 |          1 |        6 |      0 |
| Fishes         | Chondrichthyes    |               0 |         26 |        1 |      0 |
| Fishes         | Dipnoi            |               0 |          1 |        0 |      0 |
| Fishes         | Actinopterygii    |               1 |        225 |       39 |      9 |
| Fishes         | Hyperotreti       |               0 |          1 |        0 |      0 |
| Fishes         | Hyperoartia       |               0 |          4 |        0 |      0 |
| Fishes         | Coelacanthimorpha |               0 |          1 |        0 |      0 |
| Mammals        | Mammalia          |               4 |        173 |       89 |      7 |
| Birds          | Aves              |               1 |        106 |       54 |      5 |
| Amphibians     | Amphibia          |               0 |         29 |        3 |      1 |
| Ascomycetes    | Ascomycota        |              47 |         49 |      276 |    162 |
| Basidiomycetes | Basidiomycota     |              27 |         18 |       48 |     32 |
| Green Plants   | Viridiplantae     |               9 |        155 |       58 |      9 |
| Land Plants    | Embryophyta       |               7 |        152 |       53 |      8 |
| Apicomplexans  | Apicomplexa       |               2 |         25 |       39 |      3 |
| Kinetoplasts   | Kinetoplastida    |               1 |         13 |        7 |      3 |

Table: refseq - Eukaryotes

| GROUP_NAME     | SCI_NAME          | Complete Genome | Chromosome | Scaffold | Contig |
| -------------- | ----------------- | --------------: | ---------: | -------: | -----: |
| Flatworms      | Platyhelminthes   |               0 |         47 |       89 |     20 |
| Roundworms     | Nematoda          |               4 |        157 |      348 |    218 |
| Insects        | Hexapoda          |              21 |       3513 |     3389 |   2573 |
| Reptiles       | Testudines        |               1 |         59 |       50 |     10 |
| Reptiles       | Lepidosauria      |               0 |        117 |      281 |     30 |
| Reptiles       | Crocodylia        |               0 |          5 |       14 |      0 |
| Fishes         | Chondrichthyes    |               0 |         56 |       60 |      6 |
| Fishes         | Dipnoi            |               0 |          4 |        0 |      2 |
| Fishes         | Actinopterygii    |              31 |       1111 |     2107 |    320 |
| Fishes         | Hyperotreti       |               0 |          4 |        3 |      0 |
| Fishes         | Hyperoartia       |               0 |          7 |       14 |      4 |
| Fishes         | Coelacanthimorpha |               0 |          1 |        3 |      0 |
| Mammals        | Mammalia          |              25 |       1471 |     2280 |    973 |
| Birds          | Aves              |               3 |        447 |     2191 |    330 |
| Amphibians     | Amphibia          |               0 |         93 |      186 |     12 |
| Ascomycetes    | Ascomycota        |             468 |       1312 |    10872 |   6713 |
| Basidiomycetes | Basidiomycota     |             127 |        188 |     1746 |   1247 |
| Green Plants   | Viridiplantae     |             252 |       4203 |     2895 |   1261 |
| Land Plants    | Embryophyta       |             220 |       4132 |     2688 |   1024 |
| Apicomplexans  | Apicomplexa       |              20 |        132 |      199 |     89 |
| Kinetoplasts   | Kinetoplastida    |              16 |         72 |      119 |    104 |

Table: genbank - Eukaryotes

## Example 2: find accessions of a species

Staphylococcus capitis - 29388 - 头状葡萄球菌

```shell
nwr info "Staphylococcus capitis"

nwr member 29388

```

```shell
echo '
.headers ON
    SELECT
        organism_name,
        species,
        genus,
        ftp_path,
        assembly_level
    FROM ar
    WHERE 1=1
        AND tax_id != species_id    -- with strain ID
        AND species_id IN (29388)
    ' |
    sqlite3 -tabs ~/.nwr/ar_refseq.sqlite \
    > Scap.assembly.tsv

echo '
    SELECT
        species || " " || REPLACE(assembly_accession, ".", "_") AS organism_name,
        species,
        genus,
        ftp_path,
        assembly_level
    FROM ar
    WHERE 1=1
        AND tax_id = species_id     -- no strain ID
        AND assembly_level IN ("Chromosome", "Complete Genome")
        AND species_id IN (29388)
    ' |
    sqlite3 -tabs ~/.nwr/ar_refseq.sqlite \
    >> Scap.assembly.tsv

```

## Example 3: find model organisms in a family

```shell
echo "
.headers ON
    SELECT
        tax_id,
        organism_name
    FROM ar
    WHERE 1=1
        AND family IN ('Enterobacteriaceae')
        AND refseq_category IN ('reference genome')
    " |
    sqlite3 -tabs ~/.nwr/ar_refseq.sqlite |
    sed '1s/^/#/' |
    tva to md

```

| #tax_id | organism_name                                                    |
|---------|------------------------------------------------------------------|
| 511145  | Escherichia coli str. K-12 substr. MG1655                        |
| 198214  | Shigella flexneri 2a str. 301                                    |
| 99287   | Salmonella enterica subsp. enterica serovar Typhimurium str. LT2 |
| 386585  | Escherichia coli O157:H7 str. Sakai                              |
| 1125630 | Klebsiella pneumoniae subsp. pneumoniae HS11286                  |
