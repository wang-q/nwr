# NCBI Assembly Reports

- [NCBI Assembly Reports](#ncbi-assembly-reports)
    * [Preparations](#preparations)
    * [NCBI ASSEMBLY](#ncbi-assembly)
    * [Example 1: count qualified assemblies of Eukaryote groups](#example-1-count-qualified-assemblies-of-eukaryote-groups)
    * [Example 2: count qualified assemblies of Prokaryote groups](#example-2-count-qualified-assemblies-of-prokaryote-groups)
    * [Example 3: find accessions of a species](#example-3-find-accessions-of-a-species)

Download date: 2022-2-24

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

Requires SQLite version 3.34 or above.

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
| Chromosome      | Full       |   4650 |
| Chromosome      | Partial    |    435 |
| Complete Genome | Full       |  37097 |
| Complete Genome | Partial    |      1 |
| Contig          | Full       | 129386 |
| Contig          | Partial    |      1 |
| Scaffold        | Full       |  81207 |
| Scaffold        | Partial    |     28 |

Table: refseq

| assembly_level  | genome_rep |  count |
|-----------------|------------|-------:|
| Chromosome      | Full       |   8872 |
| Chromosome      | Partial    |   2190 |
| Complete Genome | Full       |  75022 |
| Complete Genome | Partial    |     23 |
| Contig          | Full       | 964665 |
| Contig          | Partial    |    845 |
| Scaffold        | Full       | 170154 |
| Scaffold        | Partial    |    358 |

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
| Insects        | Hexapoda          | 0               | 69         | 103      | 24     |
| Reptiles       | Testudines        | 0               | 7          | 3        | 0      |
| Reptiles       | Lepidosauria      | 0               | 6          | 9        | 1      |
| Reptiles       | Crocodylia        | 0               | 0          | 4        | 0      |
| Fishes         | Chondrichthyes    | 0               | 4          | 2        | 0      |
| Fishes         | Dipnoi            | 0               | 1          | 0        | 0      |
| Fishes         | Actinopterygii    | 0               | 95         | 34       | 1      |
| Fishes         | Hyperotreti       | 0               | 0          | 0        | 0      |
| Fishes         | Hyperoartia       | 0               | 1          | 0        | 0      |
| Fishes         | Coelacanthimorpha | 0               | 0          | 1        | 0      |
| Mammals        | Mammalia          | 0               | 77         | 99       | 5      |
| Birds          | Aves              | 0               | 27         | 63       | 2      |
| Amphibians     | Amphibia          | 0               | 8          | 1        | 0      |
| Ascomycetes    | Ascomycota        | 19              | 43         | 198      | 74     |
| Basidiomycetes | Basidiomycota     | 2               | 7          | 48       | 20     |
| Green Plants   | Viridiplantae     | 2               | 91         | 43       | 3      |
| Land Plants    | Embryophyta       | 0               | 88         | 38       | 2      |
| Apicomplexans  | Apicomplexa       | 4               | 21         | 14       | 2      |
| Kinetoplasts   | Kinetoplastida    | 1               | 7          | 6        | 0      |

Table: refseq - Eukaryotes

| GROUP_NAME     | SCI_NAME          | Complete Genome | Chromosome | Scaffold | Contig |
|----------------|-------------------|-----------------|------------|----------|--------|
| Flatworms      | Platyhelminthes   | 0               | 7          | 54       | 9      |
| Roundworms     | Nematoda          | 1               | 25         | 190      | 56     |
| Insects        | Hexapoda          | 0               | 472        | 1004     | 1208   |
| Reptiles       | Testudines        | 0               | 8          | 24       | 2      |
| Reptiles       | Lepidosauria      | 0               | 16         | 40       | 3      |
| Reptiles       | Crocodylia        | 0               | 0          | 8        | 0      |
| Fishes         | Chondrichthyes    | 0               | 8          | 12       | 2      |
| Fishes         | Dipnoi            | 0               | 2          | 0        | 0      |
| Fishes         | Actinopterygii    | 0               | 239        | 1059     | 68     |
| Fishes         | Hyperotreti       | 0               | 0          | 1        | 0      |
| Fishes         | Hyperoartia       | 0               | 3          | 6        | 0      |
| Fishes         | Coelacanthimorpha | 0               | 0          | 2        | 0      |
| Mammals        | Mammalia          | 1               | 383        | 1437     | 163    |
| Birds          | Aves              | 0               | 94         | 597      | 41     |
| Amphibians     | Amphibia          | 0               | 17         | 19       | 2      |
| Ascomycetes    | Ascomycota        | 164             | 874        | 4787     | 2096   |
| Basidiomycetes | Basidiomycota     | 20              | 59         | 920      | 487    |
| Green Plants   | Viridiplantae     | 5               | 729        | 776      | 334    |
| Land Plants    | Embryophyta       | 2               | 720        | 684      | 276    |
| Apicomplexans  | Apicomplexa       | 12              | 88         | 169      | 63     |
| Kinetoplasts   | Kinetoplastida    | 10              | 40         | 64       | 46     |

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
| Acidobacteria         | 26              | 1          | 20       | 23     |
| Actinobacteria        | 2486            | 511        | 9360     | 10327  |
| Aquificae             | 14              | 2          | 8        | 9      |
| Armatimonadetes       | 1               | 1          | 4        | 1      |
| Atribacterota         | 1               | 0          | 0        | 0      |
| Bacteroidetes         | 919             | 149        | 2624     | 3501   |
| Balneolaeota          | 0               | 0          | 4        | 15     |
| Caldiserica           | 1               | 0          | 0        | 0      |
| Calditrichaeota       | 1               | 1          | 0        | 0      |
| Chlamydiae            | 187             | 83         | 60       | 101    |
| Chlorobi              | 13              | 0          | 6        | 11     |
| Chloroflexi           | 4               | 0          | 5        | 4      |
| Chrysiogenetes        | 2               | 0          | 2        | 0      |
| Coprothermobacterota  | 1               | 0          | 1        | 2      |
| Cyanobacteria         | 186             | 45         | 281      | 444    |
| Deferribacteres       | 5               | 0          | 3        | 3      |
| Deinococcus-Thermus   | 63              | 3          | 54       | 131    |
| Dictyoglomi           | 2               | 0          | 0        | 1      |
| Elusimicrobia         | 2               | 0          | 0        | 1      |
| Fibrobacteres         | 2               | 0          | 10       | 28     |
| Firmicutes            | 5609            | 857        | 27179    | 34107  |
| Fusobacteria          | 76              | 5          | 107      | 132    |
| Gemmatimonadetes      | 4               | 0          | 2        | 1      |
| Ignavibacteriae       | 2               | 0          | 0        | 0      |
| Kiritimatiellaeota    | 2               | 0          | 0        | 2      |
| Lentisphaerae         | 0               | 0          | 2        | 4      |
| Nitrospinae           | 0               | 0          | 1        | 2      |
| Nitrospirae           | 9               | 0          | 3        | 10     |
| Planctomycetes        | 54              | 26         | 37       | 49     |
| Proteobacteria        | 14583           | 2147       | 39620    | 78138  |
| Rhodothermaeota       | 1               | 0          | 3        | 4      |
| Spirochaetes          | 165             | 138        | 262      | 837    |
| Synergistetes         | 5               | 4          | 10       | 20     |
| Tenericutes           | 431             | 18         | 155      | 424    |
| Thermodesulfobacteria | 7               | 0          | 4        | 5      |
| Thermotogae           | 41              | 1          | 33       | 38     |
| Verrucomicrobia       | 112             | 7          | 163      | 90     |
| Archaea               |                 |            |          |        |
| Crenarchaeota         | 93              | 9          | 10       | 71     |
| Euryarchaeota         | 277             | 8          | 248      | 405    |
| Nanoarchaeota         | 0               | 0          | 0        | 0      |
| Thaumarchaeota        | 10              | 0          | 4        | 5      |

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
    AND tax_id != species_id               -- with strain ID
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
    AND tax_id = species_id               -- no strain ID
    AND assembly_level IN ("Chromosome", "Complete Genome")
    AND species_id IN (29388)
' |
    sqlite3 -tabs ~/.nwr/ar_refseq.sqlite \
    >> Scap.assembly.tsv

```
