# NCBI Assembly Reports

<!-- toc -->

- [Preparations](#preparations)
- [NCBI ASSEMBLY](#ncbi-assembly)
- [Example 1: count qualified assemblies of Eukaryote groups](#example-1-count-qualified-assemblies-of-eukaryote-groups)
- [Example 2: count qualified assemblies of Prokaryote groups](#example-2-count-qualified-assemblies-of-prokaryote-groups)
- [Example 3: find accessions of a species](#example-3-find-accessions-of-a-species)
- [Example 4: find model organisms in a family](#example-4-find-model-organisms-in-a-family)

<!-- tocstop -->

Download date: Thu Feb 9 05:00:03 CST 2023

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
| Chromosome      | Full       |   5352 |
| Chromosome      | Partial    |    455 |
| Complete Genome | Full       |  43405 |
| Complete Genome | Partial    |     22 |
| Contig          | Full       | 149956 |
| Contig          | Partial    |      1 |
| Scaffold        | Full       |  90946 |

Table: refseq

| assembly_level  | genome_rep |   count |
|-----------------|------------|--------:|
| Chromosome      | Full       |   12079 |
| Chromosome      | Partial    |    2266 |
| Complete Genome | Full       |   90827 |
| Complete Genome | Partial    |     893 |
| Contig          | Full       | 1276924 |
| Contig          | Partial    |     851 |
| Scaffold        | Full       |  206002 |
| Scaffold        | Partial    |     324 |

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
| Roundworms     | Nematoda          | 1               | 2          | 5        | 0      |
| Insects        | Hexapoda          | 0               | 112        | 96       | 22     |
| Reptiles       | Testudines        | 0               | 9          | 3        | 0      |
| Reptiles       | Lepidosauria      | 0               | 9          | 9        | 1      |
| Reptiles       | Crocodylia        | 0               | 0          | 4        | 0      |
| Fishes         | Chondrichthyes    | 0               | 7          | 1        | 0      |
| Fishes         | Dipnoi            | 0               | 1          | 0        | 0      |
| Fishes         | Actinopterygii    | 0               | 114        | 33       | 1      |
| Fishes         | Hyperotreti       | 0               | 0          | 0        | 0      |
| Fishes         | Hyperoartia       | 0               | 1          | 0        | 0      |
| Fishes         | Coelacanthimorpha | 0               | 0          | 1        | 0      |
| Mammals        | Mammalia          | 1               | 92         | 101      | 6      |
| Birds          | Aves              | 0               | 35         | 64       | 2      |
| Amphibians     | Amphibia          | 0               | 8          | 1        | 0      |
| Ascomycetes    | Ascomycota        | 22              | 47         | 228      | 80     |
| Basidiomycetes | Basidiomycota     | 3               | 9          | 48       | 25     |
| Green Plants   | Viridiplantae     | 2               | 107        | 42       | 5      |
| Land Plants    | Embryophyta       | 0               | 104        | 37       | 4      |
| Apicomplexans  | Apicomplexa       | 4               | 21         | 14       | 2      |
| Kinetoplasts   | Kinetoplastida    | 1               | 7          | 6        | 0      |

Table: refseq - Eukaryotes

| GROUP_NAME     | SCI_NAME          | Complete Genome | Chromosome | Scaffold | Contig |
|----------------|-------------------|-----------------|------------|----------|--------|
| Flatworms      | Platyhelminthes   | 0               | 13         | 75       | 10     |
| Roundworms     | Nematoda          | 2               | 38         | 211      | 75     |
| Insects        | Hexapoda          | 0               | 829        | 1272     | 1468   |
| Reptiles       | Testudines        | 0               | 14         | 28       | 3      |
| Reptiles       | Lepidosauria      | 0               | 21         | 76       | 8      |
| Reptiles       | Crocodylia        | 0               | 0          | 8        | 0      |
| Fishes         | Chondrichthyes    | 0               | 8          | 17       | 2      |
| Fishes         | Dipnoi            | 0               | 2          | 0        | 0      |
| Fishes         | Actinopterygii    | 0               | 353        | 1307     | 102    |
| Fishes         | Hyperotreti       | 0               | 0          | 2        | 0      |
| Fishes         | Hyperoartia       | 0               | 3          | 6        | 0      |
| Fishes         | Coelacanthimorpha | 0               | 0          | 2        | 0      |
| Mammals        | Mammalia          | 1               | 333        | 1630     | 570    |
| Birds          | Aves              | 0               | 135        | 918      | 54     |
| Amphibians     | Amphibia          | 0               | 26         | 25       | 6      |
| Ascomycetes    | Ascomycota        | 222             | 933        | 6713     | 3020   |
| Basidiomycetes | Basidiomycota     | 33              | 75         | 1087     | 657    |
| Green Plants   | Viridiplantae     | 9               | 1076       | 1088     | 589    |
| Land Plants    | Embryophyta       | 3               | 1062       | 985      | 515    |
| Apicomplexans  | Apicomplexa       | 11              | 96         | 174      | 74     |
| Kinetoplasts   | Kinetoplastida    | 11              | 43         | 66       | 50     |

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
| Acidobacteria         | 30              | 1          | 30       | 37     |
| Actinomycetota        | 2954            | 558        | 10289    | 11789  |
| Aquificae             | 14              | 2          | 9        | 9      |
| Armatimonadetes       | 1               | 1          | 4        | 0      |
| Atribacterota         | 1               | 0          | 0        | 0      |
| Bacillota             | 6872            | 975        | 29778    | 39188  |
| Bacteroidota          | 1156            | 198        | 3351     | 4339   |
| Balneolota            | 1               | 0          | 6        | 17     |
| Caldiserica           | 1               | 0          | 0        | 0      |
| Calditrichaeota       | 1               | 1          | 0        | 0      |
| Chlamydiae            | 192             | 82         | 51       | 115    |
| Chlorobi              | 16              | 1          | 6        | 11     |
| Chloroflexi           | 4               | 0          | 4        | 4      |
| Chrysiogenetes        | 2               | 0          | 2        | 0      |
| Coprothermobacterota  | 1               | 0          | 1        | 2      |
| Cyanobacteria         | 226             | 50         | 276      | 594    |
| Deferribacteres       | 6               | 0          | 2        | 8      |
| Deinococcus-Thermus   | 77              | 3          | 58       | 137    |
| Dictyoglomi           | 2               | 0          | 0        | 1      |
| Elusimicrobia         | 2               | 0          | 0        | 1      |
| Fibrobacteres         | 2               | 0          | 11       | 28     |
| Fusobacteria          | 81              | 5          | 108      | 144    |
| Gemmatimonadetes      | 4               | 0          | 2        | 1      |
| Ignavibacteriae       | 2               | 0          | 0        | 0      |
| Kiritimatiellaeota    | 2               | 0          | 0        | 2      |
| Lentisphaerae         | 0               | 0          | 2        | 4      |
| Nitrospinae           | 0               | 0          | 1        | 2      |
| Nitrospirae           | 9               | 0          | 4        | 10     |
| Planctomycetota       | 54              | 27         | 40       | 64     |
| Pseudomonadota        | 18160           | 2467       | 44996    | 90796  |
| Rhodothermaeota       | 0               | 0          | 3        | 3      |
| Spirochaetes          | 352             | 149        | 275      | 936    |
| Synergistetes         | 6               | 4          | 12       | 22     |
| Tenericutes           | 485             | 28         | 168      | 458    |
| Thermodesulfobacteria | 7               | 0          | 4        | 5      |
| Thermotogae           | 42              | 1          | 33       | 38     |
| Verrucomicrobia       | 113             | 9          | 167      | 106    |
| Archaea               |                 |            |          |        |
| Crenarchaeota         | 98              | 9          | 11       | 79     |
| Euryarchaeota         | 349             | 11         | 280      | 470    |
| Nanoarchaeota         | 1               | 0          | 0        | 0      |
| Thaumarchaeota        | 13              | 0          | 4        | 9      |

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
