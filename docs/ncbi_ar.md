# NCBI Assembly Reports

<!-- TOC -->
* [NCBI Assembly Reports](#ncbi-assembly-reports)
  * [Preparations](#preparations)
  * [NCBI ASSEMBLY](#ncbi-assembly)
  * [Example 1: count qualified assemblies of Eukaryote groups](#example-1-count-qualified-assemblies-of-eukaryote-groups)
  * [Example 2: count qualified assemblies of Prokaryote groups](#example-2-count-qualified-assemblies-of-prokaryote-groups)
  * [Example 3: find accessions of a species](#example-3-find-accessions-of-a-species)
  * [Example 4: find model organisms in a family](#example-4-find-model-organisms-in-a-family)
<!-- TOC -->

Download date: Fri Nov 15 15:51:01 CST 2024

## Preparations

```shell
brew install wang-q/tap/nwr
brew install wang-q/tap/tsv-utils
brew install sqlite

# Download `taxdump.tar.gz` and assembly reports
nwr download

# Init the taxonomy database
nwr txdb

# Init the assembly databases
nwr ardb
nwr ardb --genbank

```

Requires SQLite version 3.34 or above. `sqlite` that comes with mac does not work.

## NCBI ASSEMBLY

* assembly_level

```shell
for C in refseq genbank; do
    cat ~/.nwr/assembly_summary_${C}.txt |
        sed '1d' |
        tsv-summarize -H -g assembly_level,genome_rep --count |
        keep-header -- sort |
        rgr md stdin --fmt

    echo -e "\nTable: ${C}\n\n"
done

#for C in refseq genbank; do
#    cat ~/.nwr/assembly_summary_${C}.txt |
#        sed '1d' |
#        tsv-filter -H --str-eq taxid:na --or --blank taxid
#done

```

| assembly_level  | genome_rep |   count |
|-----------------|------------|--------:|
| Chromosome      | Full       |   6,754 |
| Chromosome      | Partial    |     384 |
| Complete Genome | Full       |  59,630 |
| Complete Genome | Partial    |       7 |
| Contig          | Full       | 214,418 |
| Contig          | Partial    |       1 |
| Scaffold        | Full       | 129,131 |

Table: refseq

| assembly_level  | genome_rep |     count |
|-----------------|------------|----------:|
| Chromosome      | Full       |    24,389 |
| Chromosome      | Partial    |     1,150 |
| Complete Genome | Full       |   232,696 |
| Complete Genome | Partial    |       130 |
| Contig          | Full       | 1,999,863 |
| Contig          | Partial    |       880 |
| Scaffold        | Full       |   379,706 |
| Scaffold        | Partial    |       359 |

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
    rgr md stdin --num

```

| GROUP_NAME     | SCI_NAME          | Complete Genome | Chromosome | Scaffold | Contig |
|----------------|-------------------|----------------:|-----------:|---------:|-------:|
| Flatworms      | Platyhelminthes   |               0 |          2 |        2 |      0 |
| Roundworms     | Nematoda          |               1 |          4 |        3 |      0 |
| Insects        | Hexapoda          |               1 |        164 |       96 |     32 |
| Reptiles       | Testudines        |               0 |         11 |        3 |      1 |
| Reptiles       | Lepidosauria      |               0 |         18 |        9 |      1 |
| Reptiles       | Crocodylia        |               0 |          1 |        3 |      0 |
| Fishes         | Chondrichthyes    |               0 |         13 |        1 |      0 |
| Fishes         | Dipnoi            |               0 |          1 |        0 |      0 |
| Fishes         | Actinopterygii    |               0 |        172 |       34 |      2 |
| Fishes         | Hyperotreti       |               0 |          1 |        0 |      0 |
| Fishes         | Hyperoartia       |               0 |          2 |        0 |      0 |
| Fishes         | Coelacanthimorpha |               0 |          1 |        0 |      0 |
| Mammals        | Mammalia          |               2 |        135 |       94 |      8 |
| Birds          | Aves              |               0 |         78 |       58 |      5 |
| Amphibians     | Amphibia          |               0 |         16 |        1 |      0 |
| Ascomycetes    | Ascomycota        |              45 |         49 |      248 |    157 |
| Basidiomycetes | Basidiomycota     |              17 |         13 |       45 |     28 |
| Green Plants   | Viridiplantae     |               6 |        128 |       43 |      6 |
| Land Plants    | Embryophyta       |               4 |        125 |       38 |      5 |
| Apicomplexans  | Apicomplexa       |               4 |         23 |       21 |      3 |
| Kinetoplasts   | Kinetoplastida    |               1 |         13 |        7 |      1 |

Table: refseq - Eukaryotes

| GROUP_NAME     | SCI_NAME          | Complete Genome | Chromosome | Scaffold | Contig |
|----------------|-------------------|----------------:|-----------:|---------:|-------:|
| Flatworms      | Platyhelminthes   |               0 |         34 |       77 |     21 |
| Roundworms     | Nematoda          |               3 |        110 |      299 |    183 |
| Insects        | Hexapoda          |               7 |       1971 |     2073 |   2280 |
| Reptiles       | Testudines        |               0 |         25 |       41 |     13 |
| Reptiles       | Lepidosauria      |               0 |         67 |      189 |     27 |
| Reptiles       | Crocodylia        |               0 |          2 |       12 |      0 |
| Fishes         | Chondrichthyes    |               0 |         33 |       34 |      5 |
| Fishes         | Dipnoi            |               0 |          4 |        0 |      2 |
| Fishes         | Actinopterygii    |               4 |        663 |     1703 |    240 |
| Fishes         | Hyperotreti       |               0 |          4 |        3 |      0 |
| Fishes         | Hyperoartia       |               0 |          4 |       12 |      2 |
| Fishes         | Coelacanthimorpha |               0 |          1 |        3 |      0 |
| Mammals        | Mammalia          |              63 |       1179 |     1923 |    755 |
| Birds          | Aves              |               0 |        274 |     1409 |     99 |
| Amphibians     | Amphibia          |               0 |         65 |      122 |     17 |
| Ascomycetes    | Ascomycota        |             353 |       1119 |     9096 |   5059 |
| Basidiomycetes | Basidiomycota     |              86 |        127 |     1588 |   1052 |
| Green Plants   | Viridiplantae     |             104 |       2335 |     1998 |    997 |
| Land Plants    | Embryophyta       |              90 |       2310 |     1805 |    836 |
| Apicomplexans  | Apicomplexa       |              20 |        110 |      184 |     83 |
| Kinetoplasts   | Kinetoplastida    |              14 |         65 |       97 |     97 |

Table: genbank - Eukaryotes

## Example 2: count qualified assemblies of Prokaryote groups

```shell
echo -e "GROUP_NAME\tComplete Genome\tChromosome\tScaffold\tContig" \
    > groups.tsv

for item in Bacteria Archaea ; do
    PHYLUM=$(
        nwr member ${item} -r phylum |
            grep -v -i "Candidatus " |
            grep -v -i "candidate " |
            sed '1d' |
            cut -f 2 |
            sort
    )

    echo -e "$item\t\t\t\t"

    for P in $PHYLUM; do
        GENUS=$(
            nwr member ${P} -r genus |
                grep -v -i "Candidatus " |
                grep -v -i "candidate " |
                sed '1d' |
                cut -f 1 |
                tr "\n" "," |
                sed 's/,$/\)/' |
                sed 's/^/\(/'
        )

        if [[ ${#GENUS} -lt 3 ]]; then
            >&2 echo $P has no genera
            continue
        fi

        printf "$P\t"

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
    done
done  \
    >> groups.tsv

cat groups.tsv |
    rgr md stdin --right 2-5

```

| GROUP_NAME              | Complete Genome | Chromosome | Scaffold | Contig |
|-------------------------|----------------:|-----------:|---------:|-------:|
| Bacteria                |                 |            |          |        |
| Abditibacteriota        |               0 |          0 |        0 |      1 |
| Acidobacteriota         |              40 |          4 |       34 |     51 |
| Actinomycetota          |            4268 |        694 |    20248 |  16088 |
| Aquificota              |              17 |          2 |       16 |     20 |
| Armatimonadota          |               2 |          1 |        4 |      8 |
| Atribacterota           |               2 |          0 |        1 |      1 |
| Bacillota               |           10199 |       1259 |    37229 |  53350 |
| Bacteroidota            |            1443 |        239 |     6589 |   7576 |
| Balneolota              |               1 |          0 |       13 |     33 |
| Bdellovibrionota        |              32 |         10 |       25 |     23 |
| Caldisericota           |               1 |          0 |        0 |      0 |
| Calditrichota           |               1 |          1 |        0 |      0 |
| Campylobacterota        |            1232 |        100 |     2249 |   6825 |
| Chlamydiota             |             219 |         78 |       50 |    182 |
| Chlorobiota             |              16 |          1 |        9 |     22 |
| Chloroflexota           |              46 |          1 |       44 |     86 |
| Chrysiogenota           |               2 |          0 |        2 |      0 |
| Coprothermobacterota    |               1 |          0 |        1 |      2 |
| Cyanobacteriota         |             291 |         39 |      509 |   1056 |
| Deferribacterota        |               6 |          0 |        2 |     14 |
| Deinococcota            |              90 |          3 |       86 |    199 |
| Dictyoglomota           |               2 |          0 |        2 |      2 |
| Elusimicrobiota         |               2 |          0 |        0 |      1 |
| Fibrobacterota          |               2 |          0 |       22 |     47 |
| Fusobacteriota          |             212 |          8 |      146 |    343 |
| Gemmatimonadota         |               6 |          0 |        5 |     30 |
| Ignavibacteriota        |               3 |          0 |        2 |      8 |
| Kiritimatiellota        |               2 |          0 |        0 |      6 |
| Lentisphaerota          |               1 |          0 |        2 |     13 |
| Mycoplasmatota          |             700 |         66 |      194 |    841 |
| Myxococcota             |              63 |          5 |       31 |    133 |
| Nitrospinota            |               1 |          0 |        1 |      2 |
| Nitrospirota            |              15 |          0 |        6 |     16 |
| Planctomycetota         |              59 |         28 |       48 |     94 |
| Pseudomonadota          |           24186 |       2819 |    56463 | 118746 |
| Rhodothermota           |              13 |          3 |       38 |     94 |
| Spirochaetota           |             456 |        227 |      318 |   1103 |
| Synergistota            |               9 |          4 |       34 |     71 |
| Thermodesulfobacteriota |             133 |          9 |      162 |    300 |
| Thermodesulfobiota      |               2 |          0 |        0 |      1 |
| Thermomicrobiota        |               2 |          0 |        2 |      7 |
| Thermosulfidibacterota  |               1 |          0 |        0 |      0 |
| Thermotogota            |              43 |          1 |       65 |     60 |
| Verrucomicrobiota       |             136 |          8 |      205 |    144 |
| Vulcanimicrobiota       |               1 |          0 |        0 |      0 |
| Archaea                 |                 |            |          |        |
| Methanobacteriota       |             421 |         15 |      468 |    843 |
| Microcaldota            |               0 |          0 |        0 |      0 |
| Nanobdellota            |               1 |          0 |        0 |      0 |
| Nitrososphaerota        |              19 |          2 |       11 |     18 |
| Promethearchaeota       |               1 |          0 |        0 |      0 |
| Thermoplasmatota        |              16 |          0 |        6 |     73 |
| Thermoproteota          |             110 |          5 |       98 |    113 |

Table: refseq - Prokaryotes

| GROUP_NAME              | Complete Genome | Chromosome | Scaffold |  Contig |
|-------------------------|----------------:|-----------:|---------:|--------:|
| Bacteria                |                 |            |          |         |
| Abditibacteriota        |               0 |          0 |        1 |       8 |
| Acidobacteriota         |              41 |          4 |      112 |     383 |
| Actinomycetota          |            4645 |        760 |    26912 |   25594 |
| Aquificota              |              17 |          2 |       57 |      74 |
| Armatimonadota          |               3 |          1 |       28 |      45 |
| Atribacterota           |               2 |          0 |        3 |       3 |
| Bacillota               |           12851 |       1491 |    73555 |  340521 |
| Bacteroidota            |            1590 |        270 |    14467 |   20398 |
| Balneolota              |               1 |          2 |       37 |      88 |
| Bdellovibrionota        |              35 |         10 |      117 |     174 |
| Caldisericota           |               1 |          0 |        8 |       2 |
| Calditrichota           |               1 |          1 |        7 |      15 |
| Campylobacterota        |            2494 |        145 |     5123 |  123072 |
| Chlamydiota             |             375 |         79 |      122 |     211 |
| Chlorobiota             |              16 |          1 |       30 |      58 |
| Chloroflexota           |              50 |          1 |      244 |     294 |
| Chrysiogenota           |               2 |          0 |        2 |       0 |
| Coprothermobacterota    |               1 |          0 |       14 |       7 |
| Cyanobacteriota         |             341 |         80 |     1185 |    2784 |
| Deferribacterota        |               6 |          0 |      512 |     256 |
| Deinococcota            |              96 |          3 |      128 |     243 |
| Dictyoglomota           |               2 |          0 |        5 |       5 |
| Elusimicrobiota         |               2 |          0 |        7 |      47 |
| Fibrobacterota          |               2 |          0 |      105 |     157 |
| Fusobacteriota          |             240 |         14 |      215 |     606 |
| Gemmatimonadota         |               6 |          0 |       27 |     127 |
| Ignavibacteriota        |               3 |          1 |       51 |      37 |
| Kiritimatiellota        |               2 |          0 |        4 |      25 |
| Lentisphaerota          |               1 |          0 |       11 |      26 |
| Mycoplasmatota          |             761 |        173 |      278 |    1263 |
| Myxococcota             |              68 |          5 |       69 |     308 |
| Nitrospinota            |               1 |          0 |       13 |      63 |
| Nitrospirota            |              20 |          5 |      283 |     381 |
| Planctomycetota         |              62 |         29 |      125 |     548 |
| Pseudomonadota          |           30023 |       3604 |    96906 | 1175916 |
| Rhodothermota           |              14 |          3 |       49 |     246 |
| Spirochaetota           |             491 |        647 |      599 |    2236 |
| Synergistota            |              10 |          4 |      103 |     143 |
| Thermodesulfobacteriota |             138 |         11 |      569 |    1098 |
| Thermodesulfobiota      |               2 |          0 |        4 |       5 |
| Thermomicrobiota        |               2 |          0 |        6 |      27 |
| Thermosulfidibacterota  |               1 |          0 |        1 |       1 |
| Thermotogota            |              44 |          1 |      192 |     145 |
| Verrucomicrobiota       |             143 |          9 |     1262 |    1259 |
| Vulcanimicrobiota       |               1 |          0 |        0 |       0 |
| Archaea                 |                 |            |          |         |
| Methanobacteriota       |             449 |         19 |     1158 |    1763 |
| Microcaldota            |               0 |          0 |        0 |       0 |
| Nanobdellota            |               2 |          0 |        0 |       1 |
| Nitrososphaerota        |              21 |          5 |      145 |     548 |
| Promethearchaeota       |               1 |          0 |        0 |       6 |
| Thermoplasmatota        |              17 |          0 |       41 |     196 |
| Thermoproteota          |             120 |          5 |      431 |     325 |

Table: genbank - Prokaryotes

## Example 3: find accessions of a species

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

## Example 4: find model organisms in a family

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
    rgr md stdin

```

| #tax_id | organism_name                                                    |
|---------|------------------------------------------------------------------|
| 511145  | Escherichia coli str. K-12 substr. MG1655                        |
| 198214  | Shigella flexneri 2a str. 301                                    |
| 99287   | Salmonella enterica subsp. enterica serovar Typhimurium str. LT2 |
| 386585  | Escherichia coli O157:H7 str. Sakai                              |
| 1125630 | Klebsiella pneumoniae subsp. pneumoniae HS11286                  |
