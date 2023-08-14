# Create cladogram/phylogenetic trees from newick via xelatex/tikz/forest

## A picture is worth a thousand words

![template.png](images/template.png)

## Why not FigTree/Dendroscope/MEGA?

Full control over the trees: fonts, colors, line widths, annotations, and more.

Below is the resulting file opened in Adobe Illustrator.

This is a very clean vector graphics in which all text would be editable.

![clean graphics](images/clean-graphics.png)

## Start from scratch

* Let's start by manually entering some basal taxa of animals

```shell
cat <<'EOF' > newick/example.nwk
(Ctenophora,(Porifera,(Placozoa,(Bilateria,Cnidaria))))Animalia;
EOF

nwr indent --text '.   ' newick/example.nwk

```

* You can verify it with your eyes

```text
(
.   Ctenophora,
.   (
.   .   Porifera,
.   .   (
.   .   .   Placozoa,
.   .   .   (
.   .   .   .   Bilateria,
.   .   .   .   Cnidaria
.   .   .   )
.   .   )
.   )
)Animalia;
```

* Add more information to comments
    * One node at a time

```shell
cat newick/example.nwk |
    nwr comment stdin --node Ctenophora --comment 192 `# Add comments` |
    nwr comment stdin --node Porifera --color green --comment 8579 |
    nwr comment stdin --node Placozoa --comment 1 |
    nwr comment stdin --node Cnidaria --comment 13138 |
    nwr comment stdin --lca Bilateria,Cnidaria --label Planulozoa --dot red `# Add an internal node` |
    nwr comment stdin --lca Bilateria,Placozoa --label Parahoxozoa --color green --dot red |
    nwr comment stdin --lca Bilateria,Porifera --label NOTE --bar purple |
    nwr indent stdin |
    tee newick/example.comment.nwk
#(
#  Ctenophora[comment=192],
#  (
#    Porifera[color=green:comment=8579],
#    (
#      Placozoa[comment=1],
#      (
#        Bilateria,
#        Cnidaria[comment=13138]
#      )[label=Planulozoa:dot=red]
#    )[label=Parahoxozoa:color=green:dot=red]
#  )[label=NOTE:bar=purple]
#)Animalia;

```

* Newick file is converted to `forest` codes

```shell
nwr tex newick/example.comment.nwk --bare
#[, dot, label={Animalia}, tier=4,
#  [{Ctenophora}, comment={192}, tier=0,]
#  [, bar={purple}, label={NOTE}, tier=3,
#    [\color{green}{Porifera}, comment={8579}, tier=0,]
#    [, dot={red}, label=\color{green}{Parahoxozoa}, tier=2,
#      [{Placozoa}, comment={1}, tier=0,]
#      [, dot={red}, label={Planulozoa}, tier=1,
#        [{Bilateria}, tier=0,]
#        [{Cnidaria}, comment={13138}, tier=0,]
#      ]
#    ]
#  ]
#]

```

* Produce pdf

```shell
# Edit the .tex file as you wish
nwr tex newick/example.comment.nwk -o tex/example.tex

latexmk -xelatex tex/example.tex -outdir=pdf
latexmk -c tex/example.tex -outdir=pdf

```
