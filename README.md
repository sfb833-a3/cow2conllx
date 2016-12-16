## cow-extract

This small utility extracts tokens from the (ill-formed) COW XML format and
stores it as CoNLL-X.

Currently it extracts: 

  * Tokens.
  * Quality estimations (bdc/bpc) as features.

## Usage

`cow-extract` can read/write from stdin/stdout or files. E.g. from files:

    cow-extract somecorpus.xml somecorpus.conll

Using stdin/stdout is especially useful in combination with gzip:

    zcat somecorpus.xml.gz | cow-extract | gzip - > somecorpus.conll.gz
