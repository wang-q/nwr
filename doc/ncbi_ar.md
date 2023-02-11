# NCBI Assembly Reports

<!-- toc -->

- [Preparations](#preparations)
- [NCBI ASSEMBLY](#ncbi-assembly)
- [Example 1: count qualified assemblies of Eukaryote groups](#example-1-count-qualified-assemblies-of-eukaryote-groups)
- [Example 2: count qualified assemblies of Prokaryote groups](#example-2-count-qualified-assemblies-of-prokaryote-groups)
- [Example 3: find accessions of a species](#example-3-find-accessions-of-a-species)
- [Example 4: find model organisms in a family](#example-4-find-model-organisms-in-a-family)

<!-- tocstop -->

Download date: 2022-6-3

## Preparations

```shell
brew install wang-q/tap/nwr
brew install wang-q/tap/tsv-utils
brew install sqlite
brew install miller

# Download `taxdump.tar.gz` and assembly reports
nwr download

# Init the taxonomy database
nwr txdb

# Init the assembly database
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
        mlr --itsv --omd cat

    echo -e "\nTable: ${C}\n\n"
done

#for C in refseq genbank; do
#    cat ~/.nwr/assembly_summary_${C}.txt |
#        sed '1d' |
#        tsv-filter -H --str-eq taxid:na --or --blank taxid
#done

```

| assembly_level  | genome_rep |  count |
|-----------------|------------|-------:|
| Chromosome      | Full       |   4848 |
| Chromosome      | Partial    |    433 |
| Complete Genome | Full       |  38966 |
| Complete Genome | Partial    |      2 |
| Contig          | Full       | 134380 |
| Contig          | Partial    |      1 |
| Scaffold        | Full       |  84102 |
| Scaffold        | Partial    |     28 |

Table: refseq

| assembly_level  | genome_rep |   count |
|-----------------|------------|--------:|
| Chromosome      | Full       |    9338 |
| Chromosome      | Partial    |    2051 |
| Complete Genome | Full       |   79310 |
| Complete Genome | Partial    |     151 |
| Contig          | Full       | 1020094 |
| Contig          | Partial    |     849 |
| Scaffold        | Full       |  178781 |
| Scaffold        | Partial    |     366 |

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
    mlr --itsv --omd cat

```

| GROUP_NAME     | SCI_NAME          | Complete Genome | Chromosome | Scaffold | Contig |
|----------------|-------------------|-----------------|------------|----------|--------|
| Flatworms      | Platyhelminthes   | 0               | 1          | 3        | 0      |
| Roundworms     | Nematoda          | 1               | 2          | 5        | 0      |
| Insects        | Hexapoda          | 0               | 82         | 101      | 24     |
| Reptiles       | Testudines        | 0               | 6          | 2        | 0      |
| Reptiles       | Lepidosauria      | 0               | 6          | 9        | 1      |
| Reptiles       | Crocodylia        | 0               | 0          | 3        | 0      |
| Fishes         | Chondrichthyes    | 0               | 4          | 2        | 0      |
| Fishes         | Dipnoi            | 0               | 1          | 0        | 0      |
| Fishes         | Actinopterygii    | 0               | 99         | 33       | 1      |
| Fishes         | Hyperotreti       | 0               | 0          | 0        | 0      |
| Fishes         | Hyperoartia       | 0               | 1          | 0        | 0      |
| Fishes         | Coelacanthimorpha | 0               | 0          | 1        | 0      |
| Mammals        | Mammalia          | 1               | 82         | 100      | 5      |
| Birds          | Aves              | 0               | 28         | 63       | 2      |
| Amphibians     | Amphibia          | 0               | 8          | 1        | 0      |
| Ascomycetes    | Ascomycota        | 20              | 44         | 199      | 72     |
| Basidiomycetes | Basidiomycota     | 2               | 8          | 47       | 23     |
| Green Plants   | Viridiplantae     | 1               | 96         | 44       | 3      |
| Land Plants    | Embryophyta       | 0               | 93         | 39       | 2      |
| Apicomplexans  | Apicomplexa       | 3               | 21         | 14       | 2      |
| Kinetoplasts   | Kinetoplastida    | 1               | 7          | 6        | 0      |

Table: refseq - Eukaryotes

| GROUP_NAME     | SCI_NAME          | Complete Genome | Chromosome | Scaffold | Contig |
|----------------|-------------------|-----------------|------------|----------|--------|
| Flatworms      | Platyhelminthes   | 0               | 8          | 56       | 9      |
| Roundworms     | Nematoda          | 1               | 31         | 200      | 60     |
| Insects        | Hexapoda          | 0               | 557        | 1106     | 1284   |
| Reptiles       | Testudines        | 0               | 8          | 24       | 2      |
| Reptiles       | Lepidosauria      | 0               | 16         | 55       | 3      |
| Reptiles       | Crocodylia        | 0               | 0          | 8        | 0      |
| Fishes         | Chondrichthyes    | 0               | 8          | 12       | 2      |
| Fishes         | Dipnoi            | 0               | 2          | 0        | 0      |
| Fishes         | Actinopterygii    | 0               | 276        | 1091     | 74     |
| Fishes         | Hyperotreti       | 0               | 0          | 1        | 0      |
| Fishes         | Hyperoartia       | 0               | 3          | 6        | 0      |
| Fishes         | Coelacanthimorpha | 0               | 0          | 2        | 0      |
| Mammals        | Mammalia          | 1               | 266        | 1586     | 537    |
| Birds          | Aves              | 0               | 98         | 618      | 41     |
| Amphibians     | Amphibia          | 0               | 17         | 20       | 3      |
| Ascomycetes    | Ascomycota        | 129             | 887        | 5387     | 2381   |
| Basidiomycetes | Basidiomycota     | 24              | 59         | 966      | 573    |
| Green Plants   | Viridiplantae     | 6               | 789        | 874      | 488    |
| Land Plants    | Embryophyta       | 3               | 777        | 779      | 430    |
| Apicomplexans  | Apicomplexa       | 11              | 89         | 169      | 68     |
| Kinetoplasts   | Kinetoplastida    | 10              | 41         | 65       | 50     |

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
    mlr --itsv --omd cat

```

| GROUP_NAME            | Complete Genome | Chromosome | Scaffold | Contig |
|-----------------------|-----------------|------------|----------|--------|
| Bacteria              |                 |            |          |        |
| Abditibacteriota      | 0               | 0          | 0        | 1      |
| Acidobacteria         | 21              | 1          | 27       | 24     |
| Actinobacteria        | 2646            | 527        | 9429     | 10486  |
| Aquificae             | 14              | 2          | 8        | 9      |
| Armatimonadetes       | 1               | 1          | 4        | 1      |
| Atribacterota         | 1               | 0          | 0        | 0      |
| Bacteroidetes         | 968             | 177        | 2743     | 3587   |
| Balneolaeota          | 0               | 0          | 4        | 16     |
| Caldiserica           | 1               | 0          | 0        | 0      |
| Calditrichaeota       | 1               | 1          | 0        | 0      |
| Chlamydiae            | 189             | 83         | 52       | 105    |
| Chlorobi              | 13              | 0          | 6        | 11     |
| Chloroflexi           | 4               | 0          | 4        | 4      |
| Chrysiogenetes        | 2               | 0          | 2        | 0      |
| Coprothermobacterota  | 1               | 0          | 1        | 2      |
| Cyanobacteria         | 192             | 46         | 256      | 431    |
| Deferribacteres       | 5               | 0          | 3        | 7      |
| Deinococcus-Thermus   | 73              | 3          | 58       | 134    |
| Dictyoglomi           | 2               | 0          | 0        | 1      |
| Elusimicrobia         | 2               | 0          | 0        | 1      |
| Fibrobacteres         | 2               | 0          | 11       | 28     |
| Firmicutes            | 6051            | 911        | 27854    | 35584  |
| Fusobacteria          | 78              | 5          | 102      | 140    |
| Gemmatimonadetes      | 4               | 0          | 2        | 1      |
| Ignavibacteriae       | 2               | 0          | 0        | 0      |
| Kiritimatiellaeota    | 2               | 0          | 0        | 2      |
| Lentisphaerae         | 0               | 0          | 2        | 4      |
| Nitrospinae           | 0               | 0          | 1        | 2      |
| Nitrospirae           | 9               | 0          | 3        | 10     |
| Planctomycetes        | 54              | 26         | 37       | 49     |
| Proteobacteria        | 15603           | 2202       | 41561    | 81195  |
| Rhodothermaeota       | 0               | 0          | 3        | 3      |
| Spirochaetes          | 202             | 139        | 267      | 834    |
| Synergistetes         | 6               | 4          | 10       | 20     |
| Tenericutes           | 426             | 18         | 161      | 418    |
| Thermodesulfobacteria | 7               | 0          | 4        | 5      |
| Thermotogae           | 41              | 1          | 32       | 38     |
| Verrucomicrobia       | 112             | 7          | 158      | 96     |
| Archaea               |                 |            |          |        |
| Crenarchaeota         | 93              | 9          | 10       | 77     |
| Euryarchaeota         | 292             | 9          | 248      | 413    |
| Nanoarchaeota         | 0               | 0          | 0        | 0      |
| Thaumarchaeota        | 10              | 0          | 4        | 4      |

Table: refseq - Prokaryotes

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
    mlr --itsv --omd cat

```

| #tax_id | organism_name                                                    |
|---------|------------------------------------------------------------------|
| 511145  | Escherichia coli str. K-12 substr. MG1655                        |
| 198214  | Shigella flexneri 2a str. 301                                    |
| 99287   | Salmonella enterica subsp. enterica serovar Typhimurium str. LT2 |
| 386585  | Escherichia coli O157:H7 str. Sakai                              |
| 1125630 | Klebsiella pneumoniae subsp. pneumoniae HS11286                  |
