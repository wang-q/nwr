# Build alignments across a eukaryotic taxonomy rank

Genus *Trichoderma* as an example.

[TOC levels=2-4]: #

- [Taxon info](#taxon-info)
  - [List all ranks](#list-all-ranks)
  - [Species with assemblies](#species-with-assemblies)
- [Download all assemblies](#download-all-assemblies)
  - [Create .assembly.tsv](#create-assemblytsv)
  - [Count before download](#count-before-download)
  - [Download and check](#download-and-check)
  - [Rsync to hpcc](#rsync-to-hpcc)
- [BioSample](#biosample)
- [MinHash](#minhash)
  - [Condense branches in the minhash tree](#condense-branches-in-the-minhash-tree)
- [Count valid species and strains](#count-valid-species-and-strains)
  - [For genomic alignments](#for-genomic-alignments)
  - [For protein families](#for-protein-families)
- [Collect proteins](#collect-proteins)
- [Phylogenetics with fungi61(database 1)](#phylogenetics-with-fungi61database-1)
- [Phylogenetics with BUSCO(database 2)](#phylogenetics-with-buscodatabase-2)
  - [Find corresponding representative proteins by ](#find-corresponding-representative-proteins-by-)
  - [Domain related protein sequences](#domain-related-protein-sequences)
  - [Align and concat marker genes to create species tree](#align-and-concat-marker-genes-to-create-species-tree)
  - [The protein tree](#the-protein-tree)
- [Groups and targets](#groups-and-targets)
- [Prepare sequences for ](#prepare-sequences-for-)
- [Generate alignments](#generate-alignments)

## Taxon info

- [Trichoderma](https://www.ncbi.nlm.nih.gov/Taxonomy/Browser/wwwtax.cgi?id=5543)
- [Entrez records](http://www.ncbi.nlm.nih.gov/Taxonomy/Browser/wwwtax.cgi?mode=Info&id=5543)
- [WGS](https://www.ncbi.nlm.nih.gov/Traces/wgs/?view=wgs&search=Trichoderma) is now useless and
  can be ignored.

A nice review article about Trichoderma:

Woo, S.L. et al. Trichoderma: a multipurpose, plant-beneficial microorganism for eco-sustainable
agriculture. Nat Rev Microbiol 21, 312–326 (2023). https://doi.org/10.1038/s41579-022-00819-5

### List all ranks

There are no noteworthy classification ranks other than species.

```bash
# Count the number of different ranks in Trichoderma
nwr member Trichoderma | # List all the members of Trichoderma and its subgroups
    grep -v " sp." | # Exclude unidentified species
    tva stats -H -g rank --count | # Group by rank
    tva to md --num # Convert to markdown table，right-align numeric columns

nwr lineage Trichoderma |
    tva filter --str-ne 1:clade | # Filter out clade ranks
    tva filter --str-ne "1:no rank" | # Filter out no-rank entries
    sed -n '/kingdom\tFungi/,$p' | # Keep only lines from kingdom Fungi onward
    sed -E "s/\b(genus)\b/*\1*/"| # Highlight genus
    (echo -e '#rank\tsci_name\ttax_id' && cat) | # Add header row
    tva to md # Convert to markdown table
```

| rank     | count |
|----------|------:|
| genus    |     1 |
| species  |   560 |
| no rank  |     1 |
| strain   |    14 |
| forma    |     2 |
| varietas |     2 |

| #rank      | sci_name          | tax_id |
|------------|-------------------|--------|
| kingdom    | Fungi             | 4751   |
| subkingdom | Dikarya           | 451864 |
| phylum     | Ascomycota        | 4890   |
| subphylum  | Pezizomycotina    | 147538 |
| class      | Sordariomycetes   | 147550 |
| subclass   | Hypocreomycetidae | 222543 |
| order      | Hypocreales       | 5125   |
| family     | Hypocreaceae      | 5129   |
| *genus*    | Trichoderma       | 5543   |

### Species with assemblies

The family Hypocreaceae as outgroups.

```bash
mkdir -p ~/data/Trichoderma/summary
cd ~/data/Trichoderma/summary

# should have a valid name of genus
nwr member Hypocreaceae -r genus |
    grep -v " x " | # Exclude hybrid varieties
    sed '1d' |
    tva sort -n -k 1 \
    > genus.list.tsv

wc -l genus.list.tsv
# 20 genus.list.tsv

# From the NCBI RefSeq database, select the species within each genus of Hypocreaceae that have high-quality genomes
cat genus.list.tsv | tva select -f 1 | # Extract the first column (tax_id)
while read RANK_ID; do
    echo "
        SELECT
            species_id,
            species,
            COUNT(*) AS count
        FROM ar
        WHERE 1=1
            AND genus_id = ${RANK_ID}
            AND species NOT LIKE '% x %' -- Crossbreeding of two species
            AND genome_rep IN ('Full')
        GROUP BY species_id
        HAVING count >= 1
        " |
        sqlite3 -tabs ~/.nwr/ar_refseq.sqlite
done |
# Sort by the second column (species name)
    tva sort -k 2 \
    > RS1.tsv

# From Genebank database
cat genus.list.tsv | tva select -f 1 |
while read RANK_ID; do
    echo "
        SELECT
            species_id,
            species,
            COUNT(*) AS count
        FROM ar
        WHERE 1=1
            AND genus_id = ${RANK_ID}
            AND species NOT LIKE '% x %'
            AND genome_rep IN ('Full')
        GROUP BY species_id
        HAVING count >= 1
        " |
        sqlite3 -tabs ~/.nwr/ar_genbank.sqlite
done |
    tva sort -k 2 \
    > GB1.tsv

wc -l RS*.tsv GB*.tsv
# 12 RS1.tsv
# 97 GB1.tsv

# Calculate the total number of genome assemblies for all species in each file
for C in RS GB; do
    for N in $(seq 1 1 10); do
        if [ -e "${C}${N}.tsv" ]; then
            printf "${C}${N}\t"
            cat ${C}${N}.tsv |
                tva stats --sum 3
        fi
    done
done
# RS1	12
# GB1	269
```

## Download all assemblies

### Create .assembly.tsv

This step is pretty important

- `nwr template --help` will give the requirements for `.assembly.tsv`.
- The naming of assemblies has two aspects:
    - for program operation they are unique identifiers;
    - for researchers, they should provide taxonomic information.

If a RefSeq assembly is available, the corresponding GenBank one will not be listed

```bash
cd ~/data/Trichoderma/summary

# Export the reference genome of Saccharomyces cerevisiae,including organism_name,species,genus,ftp_path,biosample,assembly_level,assembly_accession
echo "
.headers ON
    SELECT
        *
    FROM ar
    WHERE 1=1
        AND species IN ('Saccharomyces cerevisiae')
        AND refseq_category IN ('reference genome')
    " |
    sqlite3 -tabs ~/.nwr/ar_refseq.sqlite |
    tva select -H -f organism_name,species,genus,ftp_path,biosample,assembly_level,assembly_accession \
    > raw.tsv

# refseq
# Write all the TaxIDs from RS1.tsv in one line, separated by ",", and assign it to SPECIES
SPECIES=$(
    cat RS1.tsv |
        tva select -f 1 |
        tr "\n" "," |
        sed 's/,$//'
)

# Based on the filtered RS1.tsv, extract all the assembly version information of the species with high-quality from the database, and store it in raw.tsv
# extract the samples from Trichoderma that have not been identified as species (sp. samples)
echo "
    SELECT
        species || ' ' || infraspecific_name || ' ' || assembly_accession AS name,
        species, genus, ftp_path, biosample, assembly_level,
        assembly_accession
    FROM ar
    WHERE 1=1
        AND species_id IN ($SPECIES)
        AND species NOT LIKE '% sp.%'
        AND species NOT LIKE '% x %'
        AND genome_rep IN ('Full')
    " |
    sqlite3 -tabs ~/.nwr/ar_refseq.sqlite \
    >> raw.tsv

echo "
    SELECT
        genus || ' sp. ' || infraspecific_name || ' ' || assembly_accession AS name,
        genus || ' sp.', genus, ftp_path, biosample, assembly_level,
        assembly_accession
    FROM ar
    WHERE 1=1
        AND species_id IN ($SPECIES)
        AND species LIKE '% sp.%'
        AND species NOT LIKE '% x %'
        AND genome_rep IN ('Full')
    " |
    sqlite3 -tabs ~/.nwr/ar_refseq.sqlite \
    >> raw.tsv

# Extract the selected RefSeq genome number (assembly_accession) to prevent duplication when adding GenBank data in the next step
cat raw.tsv |
    tva select -H -f "assembly_accession" \
    > rs.acc.tsv

# genbank
# gbrs_paired_asm:A "pairing pointer" in NCBI. If a GenBank assembly version (GCA) has a corresponding RefSeq assembly version (GCF), this field will record the number of GCF. If it does not have a corresponding RefSeq assembly version, this field is usually empty or points to itself
SPECIES=$(
    cat GB1.tsv |
        tva select -f 1 |
        tr "\n" "," |
        sed 's/,$//'
)

echo "
    SELECT
        species || ' ' || infraspecific_name || ' ' || assembly_accession AS name,
        species, genus, ftp_path, biosample, assembly_level,
        gbrs_paired_asm
    FROM ar
    WHERE 1=1
        AND species_id IN ($SPECIES)
        AND species NOT LIKE '% sp.%'
        AND species NOT LIKE '% x %'
        AND genome_rep IN ('Full')
    " |
    sqlite3 -tabs ~/.nwr/ar_genbank.sqlite |
# Remove the duplicate contents in gbrs_paired_asm and rs.acc.tsv
    tva join -f rs.acc.tsv -k 1 -d 7 -e \
    >> raw.tsv

echo "
    SELECT
        genus || ' sp. ' || infraspecific_name || ' ' || assembly_accession AS name,
        genus || ' sp.', genus, ftp_path, biosample, assembly_level,
        gbrs_paired_asm
    FROM ar
    WHERE 1=1
        AND species_id IN ($SPECIES)
        AND species LIKE '% sp.%'
        AND species NOT LIKE '% x %'
        AND genome_rep IN ('Full')
    " |
    sqlite3 -tabs ~/.nwr/ar_genbank.sqlite |
    tva join -f rs.acc.tsv -k 1 -d 7 -e \
    >> raw.tsv

# Deduplicate and check TSV table structure for consistent field counts
cat raw.tsv |
    tva uniq |
    tva check
#271 lines, 7 fields

# From the raw.tsv, filter, remove duplicates, and standardize the genomic assembly data related to Trichoderma → generate the abbreviation name → remove duplicates (ensuring the FTP path and abbreviation name are unique) → filter valid FTP/HTTP links → sort by species + abbreviation name → finally output Trichoderma.assembly.tsv
cat raw.tsv |
    grep -v '^#' |
    tva uniq |
    tva select -f 1-6 |
    nwr abbr -C "1,2,3" -m 3 --shortsub | # Abbreviate the 1,2,3 columns, the genus name retain at least 3 characters, the 7th field is the abbr_name
    tva uniq -H -f ftp_path |
    tva uniq -H -f 7 |
    sed '1d' |
    tva select -f 7,4,5,2,6 |
    (echo -e '#name\tftp_path\tbiosample\tspecies\tassembly_level' && cat ) |
    tva filter -H --or --str-in-fld 2:ftp --str-in-fld 2:http | # The 2 column contains download links (ftp or http)
    tva sort -H -k 4,1 \
    > Trichoderma.assembly.tsv

tva check < Trichoderma.assembly.tsv
#271 lines, 5 fields

# find potential duplicate strains or assemblies
cat Trichoderma.assembly.tsv |
    tva uniq -f 1 --repeated

cat Trichoderma.assembly.tsv |
    tva filter --str-not-in-fld 2:ftp

# Edit .assembly.tsv, remove unnecessary strains, check strain names and comment out poor assemblies.
# vim Trichoderma.assembly.tsv
#
# Save the file to another directory to prevent accidentally changing it
# cp Trichoderma.assembly.tsv ~/Scripts/genomes/assembly

# Cleaning
rm raw*.*sv
```

### Count before download

- `strains.taxon.tsv` - taxonomy info: species, genus, family, order, and class

```bash
cd ~/data/Trichoderma

# Generate three bash scripts named strains.sh, rank.sh and lineage.sh
# strains.sh - strains.taxon.tsv, species, genus, family, order, and class
# rank.sh - count species and strains
# lineage.sh - count strains
nwr template ~/data/Trichoderma/summary/Trichoderma.assembly.tsv \
    --count \
    --rank genus

# strains.taxon.tsv and taxa.tsv
# Generate the above two files, which respectively trace back from the species name to its genus, family, order, class and count the number of strain, specie, genus, family, order, and class
bash Count/strains.sh

# Convert to Markdown table format
cat Count/taxa.tsv |
    tva to md --fmt

# .lst and .count.tsv
# Generate the above two files, which respectively list all the genera and the number of species and the number of strains for each genus
bash Count/rank.sh

mv Count/genus.count.tsv Count/genus.before.tsv

cat Count/genus.before.tsv |
    tva to md --num
```

| item    | count |
|---------|------:|
| strain  |   267 |
| species |    67 |
| genus   |     7 |
| family  |     2 |
| order   |     2 |
| class   |     2 |

| genus            | #species | #strains |
|------------------|---------:|---------:|
| Cladobotryum     |        5 |        6 |
| Escovopsis       |        2 |        7 |
| Hypomyces        |        4 |        4 |
| Mycogone         |        2 |        2 |
| Saccharomyces    |        1 |        1 |
| Sphaerostilbella |        1 |        1 |
| Trichoderma      |       52 |      246 |

### Download and check

- When `aria2.sh` is interrupted, run `check.sh` before restarting
- For projects that have finished downloading, but have renamed strains, you can run `reorder.sh`
  to avoid re-downloading
    - The error placement information is recorded in `misplaced.tsv`
    - The list of files to be deleted is recorded in `remove.list`
- The parameters of `n50.sh` should be determined by the distribution of the description statistics
- `collect.sh` generates a file of type `.tsv`, which is intended to be opened by spreadsheet
  software.
    - Information of assemblies is collected from `*_assembly_report.txt` after downloading
    - **Note**: `*_assembly_report.txt` have `CRLF` at the end of the line
- `finish.sh` generates the following files:
    - `omit.lst` - no annotations - species that are excluded due to the absence of annotation
      information
    - `collect.pass.tsv` - Detailed information of the species that pass the N50 check
    - `pass.lst` - species that pass the n50 check
    - `rep.lst` - representative or reference strains
    - `counts.tsv`

```bash
cd ~/data/Trichoderma

# Generate six bash scripts named aria2.sh, check.sh, reorder.sh, n50.sh, collect.sh and finish.sh, and url.tsv
nwr template ~/data/Trichoderma/summary/Trichoderma.assembly.tsv \
    --ass

# Run:download the genomic files
bash ASSEMBLY/aria2.sh

# Check md5; create check.lst
# rm ASSEMBLY/check.lst
bash ASSEMBLY/check.sh

# Remove failed directories and re-download
bash ASSEMBLY/check.sh 2>&1 |
    grep "checksum failed" |
    sed 's/.*==> //;s/ checksum failed <==//' |
    parallel --no-run-if-empty --linebuffer -k -j 1 '
        dir=$(cat ASSEMBLY/url.tsv | tva filter --str-eq "1:{}" | tva select -f 3,1 | tr "\t" "/")
        if [[ -n "$dir" && -e "ASSEMBLY/$dir" ]]; then
            echo Remove ASSEMBLY/$dir
            rm -fr "ASSEMBLY/$dir"
        fi
    '

# # Put the misplaced directory into the right place
# bash ASSEMBLY/reorder.sh

# # This operation will delete some files in the directory, so please be careful
# cat ASSEMBLY/remove.lst |
#    parallel --no-run-if-empty --linebuffer -k -j 1 '
#        if [[ -e "ASSEMBLY/{}" ]]; then
#            echo Remove {}
#            rm -fr "ASSEMBLY/{}"
#        fi
#    '

# N50 C S (The default values are 100000, 1000, and 1000000, you can also input manually)
# LEN_N50:N50 value   N_CONTIG:contig number(C)    LEN_SUM:genomic length(S)
# create n50.tsv and n50.pass.tsv (N50 > 100000, C < 1000, S > 1000000)
bash ASSEMBLY/n50.sh 100000 1000 1000000

# Adjust parameters passed to `n50.sh`
cat ASSEMBLY/n50.tsv |
    tva filter -H --str-in-fld "name:_GCF_" |
    tva stats -H --min "N50" --max "C" --min "S" |
    tva transpose
# N50_min	579860
# C_max	533
# S_min	31700302

# Calculate the median, 10% and 90% thresholds of N50, C, S
cat ASSEMBLY/n50.tsv |
    tva stats -H --quantile "N50:0.1,0.5" --quantile "C:0.5,0.9" --quantile "S:0.1,0.5" |
    tva transpose # swap rows and columns
# N50_quantile_0.1	154774
# N50_quantile_0.5	1565434.5
# C_quantile_0.5	131
# C_quantile_0.9	873.1
# S_quantile_0.1	32335806.9
# S_quantile_0.5	37384663

# After the above steps are completed, run the following commands.

# Collect; create collect.tsv
bash ASSEMBLY/collect.sh

# After all completed
bash ASSEMBLY/finish.sh

cp ASSEMBLY/collect.pass.tsv summary/
cp ASSEMBLY/omit.lst summary/
cp ASSEMBLY/pass.lst summary/
cp ASSEMBLY/sp.lst summary/
cp ASSEMBLY/rep.lst summary/

cat ASSEMBLY/counts.tsv |
    tva to md --fmt
```

| #item            | fields | lines |
|------------------|-------:|------:|
| url.tsv          |      3 |   270 |
| check.lst        |      1 |   270 |
| collect.tsv      |     20 |   271 |
| n50.tsv          |      4 |   271 |
| n50.pass.tsv     |      4 |   246 |
| collect.pass.tsv |     23 |   246 |
| pass.lst         |      1 |   245 |
| omit.lst         |      1 |   192 |
| rep.lst          |      1 |    72 |
| sp.lst           |      1 |    32 |

### Rsync to hpcc

```bash
rsync -avP \
    ~/data/Trichoderma/ \
    wangq@202.119.37.251:data/Trichoderma

rsync -avP \
    -e 'ssh -p 8804' \
    ~/data/Trichoderma/ \
    wangq@58.213.64.36:data/Trichoderma

# rsync -avP wangq@202.119.37.251:data/Trichoderma/ ~/data/Trichoderma

# rsync -avP -e 'ssh -p 8804' wangq@58.213.64.36:data/Trichoderma/ ~/data/Trichoderma
```

## BioSample

Collect some sample data. ENA's BioSample missed many strains, so NCBI's was used.

```bash
cd ~/data/Trichoderma

# Check the system's maximum allowed number of files, and increase the current permission to that maximum value
ulimit -n `ulimit -Hn`

# Generate two bash scripts named download.sh and collect.sh, and sample.tsv
nwr template ~/data/Trichoderma/summary/Trichoderma.assembly.tsv \
    --bs

# Download background information
bash BioSample/download.sh

# Generate a TSV table - biosample.tsv(header - attributes.lst), ignore rare attributes
bash BioSample/collect.sh 10

tva check < BioSample/biosample.tsv
# 268 lines, 43 fields

cp BioSample/attributes.lst summary/
cp BioSample/biosample.tsv summary/
```

## MinHash

Estimate nucleotide divergences among strains.

- Abnormal strains
    - This [paper](https://doi.org/10.1038/s41467-018-07641-9) showed that > 95% intra-species and
      <83% inter-species ANI values.
    - If the maximum value of ANI between strains within a species is greater than *0.05*, the
      median and maximum value will be reported. Strains that cannot be linked by the median ANI,
      e.g., have no similar strains in the species, will be considered as abnormal strains.
    - It may consist of two scenarios:
        1. Wrong species identification
        2. Poor assembly quality
- Non-redundant strains
    - If the ANI value between two strains within a species is less than *0.005*, the two strains
      are considered to be redundant.
    - Need these files: representative.lst and omit.lst
- MinHash tree
    - A rough tree is generated by k-mean clustering.
- These abnormal strains should be manually checked to determine whether to include them in the
  subsequent steps.

```bash
cd ~/data/Trichoderma

# Generate four bash scripts named abnormal.sh, compute.sh, dist.sh and nr.sh, and species.tsv.
nwr template ~/data/Trichoderma/summary/Trichoderma.assembly.tsv \
    --mh \
    --parallel 8 \
    --in summary/pass.lst \
    --ani-ab 0.05 \
    --ani-nr 0.005

# For the strains in pass.lst, based on the k-mer length of 21 nt, compute assembly sketches(.msh)
bash MinHash/compute.sh

# Generate a list of non-redundant assembly IDs for each species, named NR.lst, and a list of redundant assembly IDs, named redundant.lst
bash MinHash/nr.sh

# Combine all NR.lst and redundant.lst files, remove duplicates and sort
fd --full-path "MinHash/.+/NR.lst" -X cat |
    sort |
    uniq \
    > summary/NR.lst
fd --full-path "MinHash/.+/redundant.lst" -X cat |
    sort |
    uniq \
    > summary/redundant.lst
wc -l summary/NR.lst summary/redundant.lst
#  126 summary/NR.lst
#   78 summary/redundant.lst

# Abnormal strains: select the strains within the species whose maximum ANI difference between them is greater than 0.05
bash MinHash/abnormal.sh

cat MinHash/abnormal.lst |
    tva join -e -f summary/sp.lst \
    > MinHash/tmp.lst
mv MinHash/tmp.lst summary/abnormal.lst

wc -l MinHash/abnormal.lst summary/abnormal.lst
#  22 MinHash/abnormal.lst
#  10 summary/abnormal.lst

# Distances between all selected sketches, then hierarchical clustering
cd ~/data/Trichoderma/

# Cluster according to the mash distance of 0.4
nwr template ~/data/Trichoderma/summary/Trichoderma.assembly.tsv \
    --mh \
    --parallel 8 \
    --not-in summary/redundant.lst \
    --height 0.4

# Filter out non-redundant genome Mash index files, calculate the Mash distance matrix for all genomes, cluster the genomes using R, and divide the clusters according to the distance threshold. Output the phylogenetic tree (tree.nwk) and the clustering results (groups.tsv).
bash MinHash/dist.sh
```

### Condense branches in the minhash tree

- This phylo-tree is not really formal/correct, and shouldn't be used to interpret phylogenetic
  relationships
- It is just used to find more abnormal strains

```bash
mkdir -p ~/data/Trichoderma/tree
cd ~/data/Trichoderma/tree

# nw_reroot: Set the root of the tree on Sa_cer_S288C
# nwr order: Sort the nodes of the evolutionary tree. --nd: sort them in ascending order based on the "number of descendants" of each node (the branch with fewer descendants appears earlier). --an: sort them in ascending order according to the alphabetical and numerical order of the node labels.
necom nwk reroot ../MinHash/tree.nwk -n Sa_cer_S288C |
    necom nwk order stdin --nd --an \
    > minhash.reroot.newick

# Map the species names onto the tree, merge the tree branches according to the species hierarchy, and clean up the annotation information of the tree
necom pl condense --map --taxon ../Count/strains.taxon.tsv --rank 2 \
    minhash.reroot.newick \
    > minhash.condensed.newick

# Compile the LaTeX file to generate a PDF
necom nwk to-tex minhash.condensed.newick --bl |
    tectonic - &&
    mv texput.pdf Trichoderma.minhash.pdf

# svg
necom nwk to-svg minhash.condensed.newick \
    > Trichoderma.minhash.svg
```

## Count valid species and strains

### For *genomic alignments*

```bash
cd ~/data/Trichoderma/

# Based on the NCBI Taxonomy information, the selected genomes of Trichoderma are subjected to hierarchical statistics and organization.
nwr template ~/data/Trichoderma/summary/Trichoderma.assembly.tsv \
    --count \
    --in summary/pass.lst \
    --not-in summary/abnormal.lst \
    --rank genus \
    --lineage family --lineage genus

# Generate strains.taxon.tsv(record the complete classification path of each strain) and taxa.tsv(record the quantity of each classification level)
bash Count/strains.sh

cat Count/taxa.tsv |
    tva to md --num

# Generate genus.lst(record all genus names) and genus.count.tsv(record the number of unique species and unique strains contained in each genus)
bash Count/rank.sh

cat Count/genus.count.tsv |
    tva to md --num

# Count the number of strains by the hierarchy of "genus → family → species", and select the species that meet the quantity requirement (≥ the input quantity)
bash Count/lineage.sh 1

cat Count/lineage.count.tsv |
    tva to md --num

# copy to summary/
cp Count/strains.taxon.tsv summary/genome.taxon.tsv
```

| item    | count |
|---------|------:|
| strain  |   232 |
| species |    60 |
| genus   |     6 |
| family  |     2 |
| order   |     2 |
| class   |     2 |

| genus         | #species | #strains |
|---------------|---------:|---------:|
| Cladobotryum  |        5 |        6 |
| Escovopsis    |        2 |        7 |
| Hypomyces     |        4 |        4 |
| Mycogone      |        1 |        1 |
| Saccharomyces |        1 |        1 |
| Trichoderma   |       47 |      213 |

| #family            | genus         | species                       | count |
|--------------------|---------------|-------------------------------|------:|
| Hypocreaceae       | Cladobotryum  | Cladobotryum_mycophilum       |     2 |
|                    |               | Cladobotryum_protrusum        |     1 |
|                    |               | Cladobotryum_rubrobrunnescens |     1 |
|                    |               | Cladobotryum_sp               |     1 |
|                    |               | Cladobotryum_tenue            |     1 |
|                    | Escovopsis    | Escovopsis_sp                 |     5 |
|                    |               | Escovopsis_weberi             |     2 |
|                    | Hypomyces     | Hypomyces_aurantius           |     1 |
|                    |               | Hypomyces_perniciosus         |     1 |
|                    |               | Hypomyces_rosellus            |     1 |
|                    |               | Hypomyces_semicircularis      |     1 |
|                    | Mycogone      | Mycogone_sp                   |     1 |
|                    | Trichoderma   | Trichoderma_aethiopicum       |     1 |
|                    |               | Trichoderma_afarasin          |     1 |
|                    |               | Trichoderma_afroharzianum     |     9 |
|                    |               | Trichoderma_aggressivum       |     1 |
|                    |               | Trichoderma_arundinaceum      |     4 |
|                    |               | Trichoderma_asperelloides     |     4 |
|                    |               | Trichoderma_asperellum        |    22 |
|                    |               | Trichoderma_atrobrunneum      |     1 |
|                    |               | Trichoderma_atroviride        |    19 |
|                    |               | Trichoderma_austrokoningii    |     1 |
|                    |               | Trichoderma_barbatum          |     1 |
|                    |               | Trichoderma_breve             |     1 |
|                    |               | Trichoderma_brevicrassum      |     1 |
|                    |               | Trichoderma_camerunense       |     1 |
|                    |               | Trichoderma_caribbaeum        |     1 |
|                    |               | Trichoderma_ceciliae          |     1 |
|                    |               | Trichoderma_chlorosporum      |     1 |
|                    |               | Trichoderma_citrinoviride     |     6 |
|                    |               | Trichoderma_compactum         |     1 |
|                    |               | Trichoderma_deliquescens      |     1 |
|                    |               | Trichoderma_endophyticum      |     4 |
|                    |               | Trichoderma_erinaceum         |     2 |
|                    |               | Trichoderma_evansii           |     1 |
|                    |               | Trichoderma_gamsii            |     6 |
|                    |               | Trichoderma_ghanense          |     1 |
|                    |               | Trichoderma_gracile           |     2 |
|                    |               | Trichoderma_guizhouense       |     1 |
|                    |               | Trichoderma_hamatum           |     5 |
|                    |               | Trichoderma_harzianum         |    18 |
|                    |               | Trichoderma_koningii          |     2 |
|                    |               | Trichoderma_koningiopsis      |     8 |
|                    |               | Trichoderma_lentiforme        |     1 |
|                    |               | Trichoderma_lixii             |     1 |
|                    |               | Trichoderma_longibrachiatum   |    11 |
|                    |               | Trichoderma_orchidacearum     |     1 |
|                    |               | Trichoderma_pleuroticola      |     1 |
|                    |               | Trichoderma_polysporum        |     1 |
|                    |               | Trichoderma_reesei            |    25 |
|                    |               | Trichoderma_semiorbis         |     1 |
|                    |               | Trichoderma_simmonsii         |     1 |
|                    |               | Trichoderma_sp                |    25 |
|                    |               | Trichoderma_taxi              |     1 |
|                    |               | Trichoderma_velutinum         |     1 |
|                    |               | Trichoderma_virens            |     9 |
|                    |               | Trichoderma_viride            |     4 |
|                    |               | Trichoderma_virilente         |     1 |
|                    |               | Trichoderma_yunnanense        |     1 |
| Saccharomycetaceae | Saccharomyces | Saccharomyces_cerevisiae      |     1 |

### For *protein families*

```bash
cd ~/data/Trichoderma/

# Excluded the strains listed in omit.lst
nwr template ~/data/Trichoderma/summary/Trichoderma.assembly.tsv \
    --count \
    --in summary/pass.lst \
    --not-in summary/abnormal.lst \
    --not-in summary/omit.lst \
    --rank genus

# strains.taxon.tsv and taxa.tsv
bash Count/strains.sh

cat Count/taxa.tsv |
    tva to md --num

# genus.lst and genus.count.tsv
bash Count/rank.sh

cat Count/genus.count.tsv |
    tva to md --num

# copy to summary/
cp Count/strains.taxon.tsv summary/protein.taxon.tsv
```

| item    | count |
|---------|------:|
| strain  |    67 |
| species |    34 |
| genus   |     4 |
| family  |     2 |
| order   |     2 |
| class   |     2 |

| genus         | #species | #strains |
|---------------|---------:|---------:|
| Cladobotryum  |        1 |        1 |
| Escovopsis    |        1 |        1 |
| Saccharomyces |        1 |        1 |
| Trichoderma   |       31 |       64 |

## Collect proteins

```bash
cd ~/data/Trichoderma/

nwr template ~/data/Trichoderma/summary/Trichoderma.assembly.tsv \
    --pro \
    --parallel 8 \
    --in summary/pass.lst \
    --not-in summary/omit.lst

# Build a standardized protein sequence resource library for each species: First, filter the list of strains of the target species based on the input parameters, then batch extract the protein sequences (*_protein.faa.gz) of all strains of each species, remove duplicates to generate the non-redundant protein library of that species, and simultaneously organize the protein annotations and assembly association information and compress and save it.
bash Protein/collect.sh

# Clustering:First, remove redundant proteins of the species (strain-level redundancy removal → species-level representative sequences) (95% similarity rep_seq.fa.gz), then divide them into functional families based on 80% similarity (species-level representative sequences → protein families) (fam88_cluster.tsv), and then divide them into evolutionary families based on 30% similarity (divergent protein family clustering) (fam38_cluster.tsv). Finally, obtain protein classification results at different levels
# It may need to be run several times
bash Protein/cluster.sh

rm -fr Protein/tmp/

# First, select the list of filtered strains for each species, and then integrate the protein sequences, annotations, clustering results, etc. of each species into the SQLite database (seq.sqlite).
bash Protein/info.sh

# First, select the list of filtered strains for each species, and then extract the statistical indicators(species, strain_sum......) from the seq.sqlite database of each species
bash Protein/count.sh

cat Protein/counts.tsv |
    tva stats -H --count --sum 2-7 | # Calculate the sum of the values in columns 2 to 7
    sed 's/^count/species/' |
    tva transpose |
    (echo -e "#item\tcount" && cat) |
    tva to md --fmt
```

| #item      |   count |
|------------|--------:|
| species    |      37 |
| strain_sum |      74 |
| total_sum  | 775,642 |
| dedup_sum  | 775,642 |
| rep_sum    | 552,194 |
| fam88_sum  | 484,698 |
| fam38_sum  | 406,746 |

## Phylogenetics with fungi61(database 1)

```bash
cd ~/data/Trichoderma/

mkdir -p HMM

# The Fungi HMM set
tar xvfz ~/data/HMM/fungi61/fungi61.tar.gz --directory=HMM
cp HMM/fungi61.lst HMM/marker.lst
```

## Phylogenetics with BUSCO(database 2)

```bash
cd ~/data/Trichoderma/

rm -fr BUSCO

# download busco database
curl -L https://busco-data.ezlab.org/v5/data/lineages/fungi_odb10.2024-01-08.tar.gz |
    tar xvz
mv fungi_odb10/ BUSCO

#curl -L https://busco-data.ezlab.org/v5/data/lineages/ascomycota_odb10.2024-01-08.tar.gz |
#    tar xvz
#mv ascomycota_odb10/ BUSCO
```

### Find corresponding representative proteins by `hmmsearch`

```bash
cd ~/data/Trichoderma

# Only take the species from pass.lst and exclude those species in omit.lst that have no annotations
cat Protein/species.tsv |
    tva join -f summary/pass.lst -k 1 |
    tva join -e -f summary/omit.lst -k 1 \
    > Protein/species-f.tsv

#fd --full-path "Protein/.+/busco.tsv" -X rm

# In the protein sequences of each species, find the sequences that match with BUSCO, and format the output as a BUSCO marker - protein ID mapping table
cat Protein/species-f.tsv |
    tva select -f 2 |
    tva uniq |
while read SPECIES; do
    if [[ -s Protein/"${SPECIES}"/busco.tsv ]]; then
        continue
    fi
    if [[ ! -f Protein/"${SPECIES}"/rep_seq.fa.gz ]]; then
        continue
    fi

    echo >&2 "${SPECIES}"

    cat BUSCO/scores_cutoff |
        parallel --colsep '\s+' --no-run-if-empty --linebuffer -k -j 4 "
            gzip -dcf Protein/${SPECIES}/rep_seq.fa.gz |
                hmmsearch -T {2} --domT {2} --noali --notextw BUSCO/hmms/{1}.hmm - |
                grep '>>' |
                perl -nl -e ' m(>>\s+(\S+)) and printf qq(%s\t%s\n), q({1}), \$1; '
        " \
        > Protein/${SPECIES}/busco.tsv
done

# Count the number of occurrences of each BUSCO marker, and calculate the quartiles, median, and upper quartile
fd --full-path "Protein/.+/busco.tsv" -X cat | # Integrate all busco.tsv
    tva stats --group-by 1 --count |
    tva stats --quantile 2:0.25,0.5,0.75
#40      42      45

# There are 36 species and 67 strains
# Modify based on the actual situation, keep the Markers whose occurrence frequency is between 40 and 75 times, and discard those with fewer than 40 occurrences or more than 75 occurrences (in marker.omit.lst)
fd --full-path "Protein/.+/busco.tsv" -X cat |
    tva stats --group-by 1 --count |
    tva filter --invert --ge 2:40 --le 2:75 |
    cut -f 1 \
    > Protein/marker.omit.lst

# Extract the entire list of BUSCO markers
cat BUSCO/scores_cutoff |
    parallel --colsep '\s+' --no-run-if-empty --linebuffer -k -j 1 "
        echo {1}
    " \
    > Protein/marker.lst

wc -l Protein/marker.lst Protein/marker.omit.lst
# 758 Protein/marker.lst
#   186 Protein/marker.omit.lst

# Remove the markers that need to be removed and generate a list of single-copy genes
# In the local SQLite database, establish an index between the IDs of BUSCO Markers and the actual protein sequences
cat Protein/species-f.tsv |
    tva select -f 2 |
    tva uniq |
while read SPECIES; do
    if [[ ! -s Protein/"${SPECIES}"/busco.tsv ]]; then
        continue
    fi
    if [[ ! -f Protein/"${SPECIES}"/seq.sqlite ]]; then
        continue
    fi

    echo >&2 "${SPECIES}"

    # single copy
    cat Protein/"${SPECIES}"/busco.tsv |
        grep -v -Fw -f Protein/marker.omit.lst \
        > Protein/"${SPECIES}"/busco.sc.tsv

    nwr seqdb -w Protein/${SPECIES} --rep f3=Protein/${SPECIES}/busco.sc.tsv

done
```

### Domain related protein sequences

```bash
cd ~/data/Trichoderma

mkdir -p Domain

# From the local database of 36 species, extract the protein sequences corresponding to the single-copy BUSCO genes through SQL queries
cat Protein/species-f.tsv |
    tva select -f 2 |
    tva uniq |
while read SPECIES; do
    if [[ ! -f Protein/"${SPECIES}"/seq.sqlite ]]; then
        continue
    fi

    echo >&2 "${SPECIES}"

    echo "
        SELECT
            seq.name,
            asm.name,
            rep.f3
        FROM asm_seq
        JOIN rep_seq ON asm_seq.seq_id = rep_seq.seq_id
        JOIN seq ON asm_seq.seq_id = seq.id
        JOIN rep ON rep_seq.rep_id = rep.id
        JOIN asm ON asm_seq.asm_id = asm.id
        WHERE 1=1
            AND rep.f3 IS NOT NULL
        ORDER BY
            asm.name,
            rep.f3
        " |
        sqlite3 -tabs Protein/${SPECIES}/seq.sqlite \
        > Protein/${SPECIES}/seq_asm_f3.tsv
    # Extract specific sequences from pro.fa.gz
    pgr fa some Protein/"${SPECIES}"/pro.fa.gz <(
            tva select -f 1 Protein/"${SPECIES}"/seq_asm_f3.tsv |
                tva uniq
        )
done |
    pgr fa dedup stdin |
    pgr fa gz stdin -o Domain/busco.fa.gz

fd --full-path "Protein/.+/seq_asm_f3.tsv" -X cat \
    > Domain/seq_asm_f3.tsv

# redundancy removal
cat Domain/seq_asm_f3.tsv |
    tva join -e -d 2 -f summary/redundant.lst -k 1 \
    > Domain/seq_asm_f3.NR.tsv
```

### Align and concat marker genes to create species tree

```bash
cd ~/data/Trichoderma

# For each BUSCO Marker, create a directory and extract the corresponding protein sequence
cat Protein/marker.lst |
    grep -v -Fw -f Protein/marker.omit.lst |
    parallel --no-run-if-empty --linebuffer -k -j 4 '
        echo >&2 "==> marker [{}]"

        mkdir -p Domain/{}

        pgr fa some Domain/busco.fa.gz <(
            cat Domain/seq_asm_f3.tsv |
                tva filter --str-eq "3:{}" |
                tva select -f 1 |
                tva uniq
            ) \
            > Domain/{}/{}.pro.fa
    '

# Use Mafft to perform sequence alignment for each BUSCO Marker
cat Protein/marker.lst |
    grep -v -Fw -f Protein/marker.omit.lst |
    parallel --no-run-if-empty --linebuffer -k -j 4 '
        echo >&2 "==> marker [{}]"
        if [ ! -s Domain/{}/{}.pro.fa ]; then
            exit
        fi
        if [ -s Domain/{}/{}.aln.fa ]; then
            exit
        fi

#        muscle -quiet -in Domain/{}/{}.pro.fa -out Domain/{}/{}.aln.fa
        mafft --auto Domain/{}/{}.pro.fa > Domain/{}/{}.aln.fa
    '

# Change protein names in align file to strain names
cat Protein/marker.lst |
    grep -v -Fw -f Protein/marker.omit.lst |
while read marker; do
    echo >&2 "==> marker [${marker}]"
    if [ ! -s Domain/${marker}/${marker}.pro.fa ]; then
        continue
    fi

    # sometimes `muscle` can not produce alignments
    if [ ! -s Domain/${marker}/${marker}.aln.fa ]; then
        continue
    fi

    # Only NR strains
    # 1 name to many names
    cat Domain/seq_asm_f3.NR.tsv |
        tva filter --str-eq "3:${marker}" |
        tva select -f 1-2 |
        pgr fa replace -s Domain/${marker}/${marker}.aln.fa stdin \
        > Domain/${marker}/${marker}.replace.fa
done

# Merge all the align results into one file(Domain/busco.aln.fas)
cat Protein/marker.lst |
    grep -v -Fw -f Protein/marker.omit.lst |
while read marker; do
    if [ ! -s Domain/${marker}/${marker}.pro.fa ]; then
        continue
    fi
    if [ ! -s Domain/${marker}/${marker}.aln.fa ]; then
        continue
    fi

    cat Domain/${marker}/${marker}.replace.fa

    # empty line for .fas
    echo
done \
    > Domain/busco.aln.fas

# Concatenate all the Busco Markers by strain name
cat Domain/seq_asm_f3.NR.tsv |
    cut -f 2 |
    tva uniq |
    sort |
    fasops concat Domain/busco.aln.fas stdin -o Domain/busco.aln.fa

# Trim poorly aligned regions with `TrimAl`
trimal -in Domain/busco.aln.fa -out Domain/busco.trim.fa -automated1

# Count total bases (top: original concatenated length, bottom: trimmed length)
pgr fa size Domain/busco.*.fa |
    tva uniq -f 2 |
    cut -f 2
#762750
#399438

# Informal tree, remove -fastest -noml to build a formal ML tree
FastTree -fastest -noml Domain/busco.trim.fa > Domain/busco.trim.newick
```

### The protein tree

```bash
cd ~/data/Trichoderma/tree

# (Similar to MinHash)
necom nwk reroot ../Domain/busco.trim.newick -n Sa_cer_S288C |
    necom nwk order stdin --nd --an \
    > busco.reroot.newick

necom pl condense --map -r species \
    busco.reroot.newick ../Count/species.tsv |
    necom nwk comment stdin -r "(S|member)=" |
    necom nwk comment stdin -r "^\d+$" |
    necom nwk order stdin --nd --an \
    > busco.condensed.newick

mv condensed.tsv busco.condense.tsv

necom nwk to-tex minhash.condensed.newick --bl -o Trichoderma.busco.tex

tectonic Trichoderma.busco.tex
```

## Groups and targets

Grouping criteria:

- The mash tree and the marker protein tree
- `MinHash/groups.tsv`

Target selecting criteria:

- `ASSEMBLY/collect.pass.tsv`
- Prefer Sanger sequenced assemblies
- RefSeq_category with `Representative Genome`
- Assembly_level with `Complete Genome` or `Chromosome`

Create a Bash `ARRAY` manually with a format of `group::target`.

```bash
mkdir -p ~/data/Trichoderma/taxon
cd ~/data/Trichoderma/taxon

# Select the strains with annotations and contig counts less than 100
cat ../ASSEMBLY/collect.pass.tsv |
    sed '1s/^#//' |
    tva filter -H --str-eq annotations:Yes --le C:100 |
    tva select -H -f name,Assembly_level,Genome_coverage,Sequencing_technology,N50,C \
    > potential-target.tsv

# Select the strains which assembly_level is Complete Genome or Chromosome, and contig counts less than 50
cat ../ASSEMBLY/collect.pass.tsv |
    tva filter -H --or \
        --str-eq Assembly_level:"Complete Genome" \
        --str-eq Assembly_level:"Chromosome" \
        --le C:50 |
        sed '1s/^#//' |
    tva select -H -f name,Assembly_level,Genome_coverage,Sequencing_technology,N50,C \
    > complete-genome.tsv

echo -e "#Serial\tGroup\tTarget\tCount" > group_target.tsv

# Based on the preset representative strains, identify the entire family to which they belong in the MinHash clustering, and associate their URLs
# groups according `groups.tsv`
ARRAY=(
    'C_E_H::E_web_GCA_001278495_1' # 1
    'T_afr_har::T_har_CGMCC_20739_GCA_019097725_1' # 3
    'T_asperello_asperellum::T_asperellum_FT101_GCA_020647865_1' # 5
    'T_atrov_koningio::T_atrov_IMI_206040_GCF_000171015_1' # 6
    'T_cit_lon_ree::T_ree_QM6a_GCF_000167675_1' # 7
    'T_vire::T_vire_Gv29_8_GCF_000170995_1' # 8
)

for item in "${ARRAY[@]}" ; do
# GROUP_NAME refers to the content before "::" in ARRAY
    GROUP_NAME="${item%%::*}"
# TARGET_NAME refers to the content after "::" in ARRAY
    TARGET_NAME="${item##*::}"

# SERIAL represents the group number of the TARGET_NAME strain in groups.tsv
    SERIAL=$(
        cat ../MinHash/groups.tsv |
            tva filter --str-eq 2:${TARGET_NAME} |
            tva select -f 1
    )
# GROUP_NAME represents the URLs of all strains in the group "SERIAL"
    cat ../MinHash/groups.tsv |
        tva filter --str-eq 1:${SERIAL} |
        tva select -f 2 |
        tva join -f ../ASSEMBLY/url.tsv -k 1 -a 3 \
        > ${GROUP_NAME}

    COUNT=$(cat ${GROUP_NAME} | wc -l )

    echo -e "${SERIAL}\t${GROUP_NAME}\t${TARGET_NAME}\t${COUNT}" >> group_target.tsv

done

# Custom groups
ARRAY=(
    'Trichoderma::T_ree_QM6a_GCF_000167675_1'
    'Trichoderma_reesei::T_ree_QM6a_GCF_000167675_1'
    'Trichoderma_asperellum::T_asperellum_FT101_GCA_020647865_1'
    'Trichoderma_harzianum::T_har_CGMCC_20739_GCA_019097725_1'
    'Trichoderma_atroviride::T_atrov_IMI_206040_GCF_000171015_1'
    'Trichoderma_virens::T_vire_Gv29_8_GCF_000170995_1'
)

SERIAL=100
for item in "${ARRAY[@]}" ; do
    GROUP_NAME="${item%%::*}"
    TARGET_NAME="${item##*::}"

    SERIAL=$((SERIAL + 1))
    GROUP_NAME_2=$(echo $GROUP_NAME | tr "_" " ") # change "_" to " "

# When GROUP_NAME is Trichoderma, collect the URLs for the "reference genome" or "representative genome" strains
    if [ "$GROUP_NAME" = "Trichoderma" ]; then
        cat ../ASSEMBLY/collect.pass.tsv |
            tva filter -H --not-blank RefSeq_category | # Extract all the strains that have been labeled as "reference genome" or "representative genome" by NCBI
            sed '1d' |
            tva select -f 1 \
            > T.tmp
        echo "C_pro_CCMJ2080_GCA_004303015_1" >> T.tmp
        echo "E_web_EWB_GCA_003055145_1" >> T.tmp
        echo "E_web_GCA_001278495_1" >> T.tmp
        echo "H_perniciosus_HP10_GCA_008477525_1" >> T.tmp
        echo "H_ros_CCMJ2808_GCA_011799845_1" >> T.tmp
        cat T.tmp |
            tva uniq |
            tva join -f ../ASSEMBLY/url.tsv -k 1 -a 3 \
            > ${GROUP_NAME}

# otherwise, collect the URLs of all the strains of the mentioned species
    else
        cat ../ASSEMBLY/collect.pass.tsv |
            tva select -f 1,2 |
            grep "${GROUP_NAME_2}" |
            tva select -f 1 |
            tva join -f ../ASSEMBLY/url.tsv -k 1 -a 3 \
            > ${GROUP_NAME}
    fi

    COUNT=$(cat ${GROUP_NAME} | wc -l )

    echo -e "${SERIAL}\t${GROUP_NAME}\t${TARGET_NAME}\t${COUNT}" >> group_target.tsv

done

cat group_target.tsv |
    tva to md --right 4 # Set the content in 4 column to be right-aligned
```

| #Serial | Group                  | Target                             | Count |
|---------|------------------------|------------------------------------|------:|
| 1       | C_E_H                  | E_web_GCA_001278495_1              |     5 |
| 3       | T_afr_har              | T_har_CGMCC_20739_GCA_019097725_1  |    20 |
| 5       | T_asperello_asperellum | T_asperellum_FT101_GCA_020647865_1 |    16 |
| 6       | T_atrov_koningio       | T_atrov_IMI_206040_GCF_000171015_1 |    15 |
| 7       | T_cit_lon_ree          | T_ree_QM6a_GCF_000167675_1         |    25 |
| 8       | T_vire                 | T_vire_Gv29_8_GCF_000170995_1      |     9 |
| 101     | Trichoderma            | T_ree_QM6a_GCF_000167675_1         |    32 |
| 102     | Trichoderma_reesei     | T_ree_QM6a_GCF_000167675_1         |    13 |
| 103     | Trichoderma_asperellum | T_asperellum_FT101_GCA_020647865_1 |    13 |
| 104     | Trichoderma_harzianum  | T_har_CGMCC_20739_GCA_019097725_1  |    10 |
| 105     | Trichoderma_atroviride | T_atrov_IMI_206040_GCF_000171015_1 |     7 |
| 106     | Trichoderma_virens     | T_vire_Gv29_8_GCF_000170995_1      |     8 |

## Prepare sequences for `egaz`

- `--perseq` for Chromosome-level assemblies and targets
    - means split fasta by names, targets or good assembles should set it

```bash
cd ~/data/Trichoderma

# /share/home/wangq/homebrew/Cellar/repeatmasker@4.1.1/4.1.1/libexec/famdb.py \
#   -i /share/home/wangq/homebrew/Cellar/repeatmasker@4.1.1/4.1.1/libexec/Libraries/RepeatMaskerLib.h5 \
#   lineage Fungi

# prep:before alignment, standardize the format, eliminate duplicate sequences, and split the long sequences into approximately 5 Mb segments
egaz template \
    ASSEMBLY \
    --prep -o Genome \
    $( cat taxon/group_target.tsv |
        sed -e '1d' | cut -f 3 |
        parallel -j 1 echo " --perseq {} "
    ) \
    $( cat taxon/complete-genome.tsv |
        sed '1d' | cut -f 1 |
        parallel -j 1 echo " --perseq {} "
    ) \
    --min 5000 --about 5000000 \
    -v --repeatmasker "--parallel 16"

bash Genome/0_prep.sh

# gff:Search for the annotation files of the strains in group_target.tsv and potential-target.tsv, and rename them uniformly as chr.gff
for n in \
    $(cat taxon/group_target.tsv | sed -e '1d' | cut -f 3 ) \
    $( cat taxon/potential-target.tsv | sed -e '1d' | cut -f 1 ) \
    ; do
    FILE_GFF=$(find ASSEMBLY -type f -name "*_genomic.gff.gz" | grep "${n}")
    echo >&2 "==> Processing ${n}/${FILE_GFF}"

    gzip -dc ${FILE_GFF} > Genome/${n}/chr.gff
done
```

## Generate alignments

```bash
cd ~/data/Trichoderma

# In each group, all the strains are compared pairwise with the target strain
# Merge all the pairwise comparison results into a multi-sequence alignment matrix based on the MinHash tree
cat taxon/group_target.tsv |
    sed -e '1d' |
    parallel --colsep '\t' --no-run-if-empty --linebuffer -k -j 1 '
        echo -e "==> Group: [{2}]\tTarget: [{3}]\n"

        egaz template \
            Genome/{3} \
            $(cat taxon/{2} | cut -f 1 | grep -v -x "{3}" | xargs -I[] echo "Genome/[]") \
            --multi -o groups/{2}/ \
            --tree MinHash/tree.nwk \
            --parallel 16 -v

        bash groups/{2}/1_pair.sh
        bash groups/{2}/3_multi.sh
    '

# clean
find groups -mindepth 1 -maxdepth 3 -type d -name "*_raw" | parallel -r rm -fr
find groups -mindepth 1 -maxdepth 3 -type d -name "*_fasta" | parallel -r rm -fr
find . -mindepth 1 -maxdepth 3 -type f -name "output.*" | parallel -r rm
```
