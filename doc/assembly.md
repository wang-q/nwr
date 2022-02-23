# NCBI Assembly Reports

Downloading date: 2022-2-24

## `txdb`

```shell
nwr download
nwr txdb

```

## ASSEMBLY

* Download

```shell
wget -N -P ~/.nwr https://ftp.ncbi.nlm.nih.gov/genomes/ASSEMBLY_REPORTS/assembly_summary_refseq.txt
wget -N -P ~/.nwr https://ftp.ncbi.nlm.nih.gov/genomes/ASSEMBLY_REPORTS/assembly_summary_genbank.txt

```

* `assembly_summary_*.txt` files have 23 tab-delimited columns

```text
assembly_accession
bioproject
biosample
wgs_master
refseq_category
taxid
species_taxid
organism_name
infraspecific_name
isolate
version_status
assembly_level
release_type
genome_rep
seq_rel_date
asm_name
submitter
gbrs_paired_asm
paired_asm_comp
ftp_path
excluded_from_refseq
relation_to_type_material
asm_not_live_date
```

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

## Create databases

I use the following columns:

```text
taxid
organism_name
bioproject
assembly_accession
wgs_master
refseq_category
assembly_level
genome_rep
seq_rel_date
asm_name
ftp_path
```

Sort and filter records

* Sort by assembly_level and seq_rel_date
* Remove incompetent strains
* Transform to SQLite Date format

```shell
for C in refseq genbank; do
    >&2 echo "==> ${C}"
    
    for L in 'Complete Genome' 'Chromosome' 'Scaffold' 'Contig'; do
        cat ~/.nwr/assembly_summary_${C}.txt |
            sed '1d' | #head -n 50 |
            sed '1s/# //' |
            tsv-select -H -f taxid,organism_name,bioproject,assembly_accession,wgs_master,refseq_category,assembly_level,genome_rep,seq_rel_date,asm_name,ftp_path |
            tsv-filter -H --str-eq assembly_level:"${L}" |
            tsv-filter -H --not-iregex organism_name:"\bbacterium\b" |
            tsv-filter -H --not-iregex organism_name:"\buncultured\b" |
            tsv-filter -H --not-iregex organism_name:"\bCandidatus\b" |
            tsv-filter -H --not-iregex organism_name:"\bunidentified\b" |
            tsv-filter -H --not-iregex organism_name:"\bmetagenome\b" |
            tsv-filter -H --not-iregex organism_name:"\barchaeon\b" |
            tsv-filter -H --not-iregex organism_name:"virus\b" |
            tsv-filter -H --not-iregex organism_name:"phage\b" |
            keep-header -- tsv-sort -k9,9 |
            perl -nla -F"\t" -e '$F[8] =~ s/\//-/g; print join qq{\t}, @F' | # Date
            sed '1s/^/#/' |
            tsv-filter -H --invert --str-eq assembly_level:Scaffold --str-eq genome_rep:Partial |
            tsv-filter -H --invert --str-eq assembly_level:Contig --str-eq genome_rep:Partial |
            nwr append stdin -r species -r genus --id;
    done |
    tsv-uniq \
    > ~/.nwr/ar_${C}.tsv

done

cat ~/.nwr/ar_refseq.tsv |
    tsv-summarize -H -g species --count |
    tsv-filter -H --gt 2:1 | 
    keep-header -- tsv-sort -k2,2nr |
    head -n 20

```

```shell
# DDL
for C in refseq genbank; do
    sqlite3 ~/.nwr/ar_${C}.sqlite <<EOF
PRAGMA journal_mode = OFF;
PRAGMA synchronous = 0;
PRAGMA cache_size = 1000000;
PRAGMA locking_mode = EXCLUSIVE;
PRAGMA temp_store = MEMORY;
DROP TABLE IF EXISTS ar;

CREATE TABLE IF NOT EXISTS ar (
    tax_id             INTEGER,
    organism_name      VARCHAR (50),
    bioproject         VARCHAR (50),
    assembly_accession VARCHAR (50),
    wgs_master         VARCHAR (50),
    refseq_category    VARCHAR (50),
    assembly_level     VARCHAR (50),
    genome_rep         VARCHAR (50),
    seq_rel_date       DATE,
    asm_name           VARCHAR (255),
    ftp_path           VARCHAR (255),
    species            VARCHAR (50),
    species_id         INTEGER,
    genus              VARCHAR (50),
    genus_id           INTEGER
);

CREATE INDEX idx_ar_tax_id ON ar(tax_id);
CREATE INDEX idx_ar_organism_name ON ar(organism_name);
CREATE INDEX idx_ar_species_id ON ar(species_id);
CREATE INDEX idx_ar_genus_id ON ar(genus_id);

EOF
done

# import
# sqlite .import doesn't accept relative paths
pushd ~/.nwr/

sqlite3 -tabs ar_refseq.sqlite <<EOF
PRAGMA journal_mode = OFF;
.import --skip 1 ar_refseq.tsv ar

EOF

sqlite3 -tabs ~/.nwr/ar_genbank.sqlite <<EOF
PRAGMA journal_mode = OFF;
.import --skip 1 ar_genbank.tsv ar

EOF

popd

```

## Example: find accessions of a species

Staphylococcus capitis - 29388 - 头状葡萄球菌

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
