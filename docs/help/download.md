# download

Behavior:

* Downloads the latest releases of `taxdump` and assembly reports from NCBI.
* Automatically verifies MD5 checksum for taxdump.
* Extracts taxdump.tar.gz to the NWR directory.
* Skips downloading if files already exist.

Manual Download:

You can also download the files manually:

```bash
mkdir -p ~/.nwr

# taxdump
wget -N -P ~/.nwr https://ftp.ncbi.nlm.nih.gov/pub/taxonomy/taxdump.tar.gz
wget -N -P ~/.nwr https://ftp.ncbi.nlm.nih.gov/pub/taxonomy/taxdump.tar.gz.md5

# assembly reports
wget -N -P ~/.nwr https://ftp.ncbi.nlm.nih.gov/genomes/ASSEMBLY_REPORTS/assembly_summary_refseq.txt
wget -N -P ~/.nwr https://ftp.ncbi.nlm.nih.gov/genomes/ASSEMBLY_REPORTS/assembly_summary_genbank.txt

# with aria2
cat <<EOF > download.txt
https://ftp.ncbi.nlm.nih.gov/pub/taxonomy/taxdump.tar.gz
https://ftp.ncbi.nlm.nih.gov/pub/taxonomy/taxdump.tar.gz.md5
https://ftp.ncbi.nlm.nih.gov/genomes/ASSEMBLY_REPORTS/assembly_summary_refseq.txt
https://ftp.ncbi.nlm.nih.gov/genomes/ASSEMBLY_REPORTS/assembly_summary_genbank.txt

EOF

aria2c -x 4 -s 2 -c -d ~/.nwr -i download.txt
```

Examples:

1. Download with default settings
   `nwr download`

2. Use a different FTP host
   `nwr download --host ftp.ncbi.nih.gov:21`

3. Custom paths
   `nwr download --tx /pub/taxonomy --ar /genomes/ASSEMBLY_REPORTS`
