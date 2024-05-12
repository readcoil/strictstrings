# strictstrings
Configurable string extraction and processing tool.  
Performs the following filtering techniques:
* Minimum and maximum string length
* Language detection
* Missing-whitespace
* Impossible ngrams
* Levenshtein similarity filtering

Most of the filtering techniques are configurable base on the command line arguments.


## Building:
git clone git@github.com:readcoil/strictstrings_dev.git \
cd strictstrings_dev \
cargo build --release \

## Running:
```
> cp ./target/release/strictstrings /usr/local/bin/strictstrings
> strictstrings --help

Performs strict filtering on strings within a file contents.

USAGE:
    strictstrings [OPTIONS] <infile>

ARGS:
    <infile>    Input file to process

OPTIONS:
    -b, --bytes                 Print byte representation after strings
    -h, --help                  Print help information
    -l, --logs <logdir>         Output filtered values to log directory
    -m, --min <MIN>             Minimum length of strings to process [default: 6]
    -M, --max <MAX>             Maximum length of strings to process [default: 200]
    -o, --out <outfile>         Output file write filtered strings
    -q, --quiet                 Silences all output
    -s, --similarity <FLOAT>    Sets a custom similarity filtering threshold [default: 0.8]
    -t, --language <FLOAT>      Sets a custom language detection threshold [default: 0.5]
    -V, --version               Print version information
    -W, --wslen <wslen>         Maximum length of strings without whitespace [default: 30]

```

Base usage:
```
> strictstrings binary.file
```
Adjusting thresholds:
```
> strictstrings -s 0.9 -t 0.6 binary.file
```

