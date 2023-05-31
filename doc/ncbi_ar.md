# NCBI Assembly Reports

<!-- toc -->

- [Preparations](#preparations)
- [NCBI ASSEMBLY](#ncbi-assembly)
- [Example 1: count qualified assemblies of Eukaryote groups](#example-1-count-qualified-assemblies-of-eukaryote-groups)
- [Example 2: count qualified assemblies of Prokaryote groups](#example-2-count-qualified-assemblies-of-prokaryote-groups)
- [Example 3: find accessions of a species](#example-3-find-accessions-of-a-species)
- [Example 4: find model organisms in a family](#example-4-find-model-organisms-in-a-family)

<!-- tocstop -->

Download date: Wed May 31 20:14:22 CST 2023

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
        mlr --itsv --omd cat |
        sed 's/-\s*|$/-:|/'

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
| Chromosome      | Full       |   5636 |
| Chromosome      | Partial    |    915 |
| Complete Genome | Full       |  47949 |
| Complete Genome | Partial    |     25 |
| Contig          | Full       | 157438 |
| Contig          | Partial    |      1 |
| Scaffold        | Full       |  93588 |

Table: refseq

| assembly_level  | genome_rep |   count |
|-----------------|------------|--------:|
| Chromosome      | Full       |   14687 |
| Chromosome      | Partial    |    2139 |
| Complete Genome | Full       |   95773 |
| Complete Genome | Partial    |     911 |
| Contig          | Full       | 1344663 |
| Contig          | Partial    |     858 |
| Scaffold        | Full       |  218122 |
| Scaffold        | Partial    |     328 |

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
| Flatworms      | Platyhelminthes   | 0               | 2          | 2        | 0      |
| Roundworms     | Nematoda          | 1               | 3          | 4        | 0      |
| Insects        | Hexapoda          | 0               | 128        | 97       | 23     |
| Reptiles       | Testudines        | 0               | 10         | 3        | 0      |
| Reptiles       | Lepidosauria      | 0               | 10         | 9        | 1      |
| Reptiles       | Crocodylia        | 0               | 0          | 4        | 0      |
| Fishes         | Chondrichthyes    | 0               | 8          | 1        | 0      |
| Fishes         | Dipnoi            | 0               | 1          | 0        | 0      |
| Fishes         | Actinopterygii    | 0               | 125        | 34       | 2      |
| Fishes         | Hyperotreti       | 0               | 0          | 0        | 0      |
| Fishes         | Hyperoartia       | 0               | 1          | 0        | 0      |
| Fishes         | Coelacanthimorpha | 0               | 0          | 1        | 0      |
| Mammals        | Mammalia          | 1               | 100        | 101      | 7      |
| Birds          | Aves              | 0               | 47         | 60       | 3      |
| Amphibians     | Amphibia          | 0               | 10         | 1        | 0      |
| Ascomycetes    | Ascomycota        | 30              | 47         | 225      | 82     |
| Basidiomycetes | Basidiomycota     | 4               | 9          | 48       | 25     |
| Green Plants   | Viridiplantae     | 2               | 108        | 44       | 5      |
| Land Plants    | Embryophyta       | 0               | 105        | 39       | 4      |
| Apicomplexans  | Apicomplexa       | 4               | 21         | 14       | 2      |
| Kinetoplasts   | Kinetoplastida    | 1               | 7          | 6        | 0      |

Table: refseq - Eukaryotes

| GROUP_NAME     | SCI_NAME          | Complete Genome | Chromosome | Scaffold | Contig |
|----------------|-------------------|-----------------|------------|----------|--------|
| Flatworms      | Platyhelminthes   | 0               | 29         | 60       | 11     |
| Roundworms     | Nematoda          | 2               | 42         | 213      | 83     |
| Insects        | Hexapoda          | 0               | 1078       | 1368     | 1602   |
| Reptiles       | Testudines        | 0               | 15         | 28       | 3      |
| Reptiles       | Lepidosauria      | 0               | 25         | 91       | 10     |
| Reptiles       | Crocodylia        | 0               | 1          | 10       | 0      |
| Fishes         | Chondrichthyes    | 0               | 11         | 17       | 2      |
| Fishes         | Dipnoi            | 0               | 2          | 0        | 0      |
| Fishes         | Actinopterygii    | 0               | 388        | 1439     | 113    |
| Fishes         | Hyperotreti       | 0               | 0          | 2        | 0      |
| Fishes         | Hyperoartia       | 0               | 3          | 8        | 0      |
| Fishes         | Coelacanthimorpha | 0               | 0          | 2        | 0      |
| Mammals        | Mammalia          | 1               | 438        | 1694     | 584    |
| Birds          | Aves              | 0               | 147        | 953      | 59     |
| Amphibians     | Amphibia          | 0               | 30         | 29       | 6      |
| Ascomycetes    | Ascomycota        | 245             | 943        | 6940     | 3552   |
| Basidiomycetes | Basidiomycota     | 49              | 83         | 1104     | 701    |
| Green Plants   | Viridiplantae     | 11              | 1216       | 1250     | 674    |
| Land Plants    | Embryophyta       | 5               | 1202       | 1131     | 599    |
| Apicomplexans  | Apicomplexa       | 13              | 97         | 174      | 75     |
| Kinetoplasts   | Kinetoplastida    | 12              | 43         | 66       | 51     |

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

| GROUP_NAME              | Complete Genome | Chromosome | Scaffold | Contig |
|-------------------------|-----------------|------------|----------|--------|
| Bacteria                |                 |            |          |        |
| Abditibacteriota        | 0               | 0          | 0        | 1      |
| Acidobacteriota         | 30              | 1          | 30       | 37     |
| Actinomycetota          | 3116            | 592        | 10338    | 12291  |
| Aquificota              | 15              | 2          | 9        | 11     |
| Armatimonadota          | 1               | 1          | 4        | 0      |
| Atribacterota           | 1               | 0          | 0        | 0      |
| Bacillota               | 7518            | 1027       | 31252    | 41276  |
| Bacteroidota            | 1192            | 197        | 3530     | 4749   |
| Balneolota              | 1               | 0          | 6        | 17     |
| Bdellovibrionota        | 19              | 2          | 9        | 11     |
| Caldisericota           | 1               | 0          | 0        | 0      |
| Calditrichota           | 1               | 1          | 0        | 0      |
| Campylobacterota        | 969             | 100        | 1701     | 5275   |
| Chlamydiota             | 192             | 82         | 31       | 121    |
| Chlorobiota             | 16              | 1          | 7        | 11     |
| Chloroflexota           | 40              | 0          | 22       | 39     |
| Chrysiogenota           | 2               | 0          | 2        | 0      |
| Coprothermobacterota    | 1               | 0          | 1        | 2      |
| Cyanobacteriota         | 225             | 50         | 268      | 608    |
| Deferribacterota        | 6               | 0          | 2        | 10     |
| Deinococcota            | 79              | 3          | 61       | 142    |
| Dictyoglomota           | 2               | 0          | 1        | 2      |
| Elusimicrobiota         | 2               | 0          | 0        | 1      |
| Fibrobacterota          | 2               | 0          | 11       | 31     |
| Fusobacteriota          | 87              | 5          | 112      | 157    |
| Gemmatimonadota         | 4               | 0          | 3        | 1      |
| Ignavibacteriota        | 2               | 0          | 0        | 0      |
| Kiritimatiellota        | 2               | 0          | 0        | 2      |
| Lentisphaerota          | 1               | 0          | 2        | 8      |
| Mycoplasmatota          | 522             | 33         | 175      | 540    |
| Myxococcota             | 47              | 5          | 29       | 117    |
| Nitrospinota            | 1               | 0          | 1        | 2      |
| Nitrospirota            | 9               | 0          | 4        | 13     |
| Planctomycetota         | 56              | 27         | 42       | 64     |
| Pseudomonadota          | 18234           | 2422       | 44022    | 89395  |
| Rhodothermota           | 12              | 2          | 37       | 88     |
| Spirochaetota           | 364             | 184        | 271      | 968    |
| Synergistota            | 7               | 4          | 14       | 32     |
| Thermodesulfobacteriota | 116             | 9          | 129      | 145    |
| Thermomicrobiota        | 2               | 0          | 1        | 4      |
| Thermotogota            | 43              | 1          | 33       | 41     |
| Verrucomicrobiota       | 117             | 9          | 166      | 108    |
| Archaea                 |                 |            |          |        |
| Euryarchaeota           | 376             | 13         | 307      | 501    |
| Nanoarchaeota           | 1               | 0          | 0        | 0      |
| Nitrososphaerota        | 13              | 0          | 4        | 10     |
| Thermoproteota          | 96              | 9          | 15       | 80     |

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
