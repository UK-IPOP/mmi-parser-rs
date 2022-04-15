# `mmi-parser`

`mmi-parser` is a rust command line tool (crate) for parsing out Fielded MetaMap Indexing (MMI) output from the National Library of Medicine's (NLM) [MetaMap tool](https://lhncbc.nlm.nih.gov/ii/tools/MetaMap.html) into jsonlines data.

The primary reference for the Fielded MMI output can be found [here](https://lhncbc.nlm.nih.gov/ii/tools/MetaMap/Docs/MMI_Output_2016.pdf).

> ! Requires MetaMap 2016 _(or newer)_ due to changes in MMI formatting !

- [`mmi-parser`](#mmi-parser)
  - [Description](#description)
    - [Justification](#justification)
  - [Requires](#requires)
  - [Installation](#installation)
  - [Usage](#usage)
    - [Brief MetaMap Intro](#brief-metamap-intro)
    - [mmi-parser (CLI)](#mmi-parser-cli)
    - [mmi-parser (API)](#mmi-parser-api)
  - [Support](#support)
  - [Contributing](#contributing)
  - [MIT License](#mit-license)

## Description

### Justification

Due to the (relatively) technical nature of running the MetaMap program (locally requires command line familiarity), it is assumed users will also be able to install and work with other command line tools (i.e. cargo).

This project uses [Rust](https://www.rust-lang.org) to parse the Fielded MMI output into
[jsonlines](https://jsonlines.org) annotated data. While not entirely a different structure, MMI was chosen as the input and jsonlines was chosen as the output for a few reasons.

MMI is by far the most dense/compressed **human-readable** version of MetaMap output, so it makes logical sense to use as input to the parser.

MMI output is often put into separate `.txt` files for each record being run through MetaMap. MMI output also contains one line per concept found. Jsonlines allows us to keep this 1:1 ratio. Each input `.txt` file will have _exactly one_ jsonlines output file with `_parsed` suffixed to the file name to clarify it is parsed output. Jsonlines also has the added benefit of maintaining the 1:1 (concept:line) ratio that the original MMI output has. Thus each jsonline can be tracked to a line in the source (MMI output) text file. This helps with tracing results. Jsonlines, compared to traditional json, also allows file buffer reading which can be a benefit when scanning large files. Finally, while MMI already has fields _associated_ with various parts of the text, jsonlines makes these _implicit_ associations **explicit** in field names.

For example:

- `data/sample.txt` --> `data/sample_parsed.jsonl`
  Where the first line in `data/sample_parsed.jsonl` will represent the first (or last depending on MetaMap) construct found in the source text document but will **always** match the first line in `data/sample.txt`.

> It is worth noting that some MetaMap pipelines produce `.txt` files with a header line indicating when the file was written. Please remove these lines BEFORE running this tool.

If you need an alternative output, perhaps for a non-technical researcher, I recommend looking at [jq](https://stedolan.github.io/jq/).

## Requires

- [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html) package manager (rust toolchain)
- [just](https://github.com/casey/just) (optional dev-dependency if you clone this repo)

## Installation

Cargo is available as a part of the Rust toolchain and is readily available via curl + sh combo (see [here](https://doc.rust-lang.org/cargo/getting-started/installation.html)).

To install the mmi-parser application, utilize cargo:

```bash
cargo install mmi-parser
```

If you also need MetaMap installed you can find instructions on how to do so [here](https://lhncbc.nlm.nih.gov/ii/tools/MetaMap/documentation/Installation.html).

There is also an API available on crates.io [here](https://crates.io/crates/mmi-parser). The scope of this API is limited to reduce maintenance burden as the primary goal of this project was an executable parser.

The API can be installed to your local Rust project by simply adding the crate to your `Cargo.toml`:

```toml
mmi-parser = "1.1.0"
```

## Usage

### Brief MetaMap Intro

Usage of MetaMap can be found extensively documented on the NLM's website or more directly in [this](https://lhncbc.nlm.nih.gov/ii/tools/MetaMap/Docs/MM_2016_Usage.pdf) document.

For our use case, we are going to assume you use a command similar in functionality to:

```bash
echo lung cancer | metamap -N > metamap_results.txt
```

The `-N` flag here is important as it signals ot MetaMap to use the MMI output.

This should result in an output similar to the following:

```bash
   1   │ /home/nanthony/public_mm/bin/SKRrun.20 /home/nanthony/public_mm/bin/metamap20.BINARY.Linux --lexicon db -Z 2020AA
       │  -N
   2   │ USER|MMI|5.18|Carcinoma of lung|C0684249|[neop]|["LUNG CANCER"-tx-1-"lung cancer"-noun-0]|TX|0/11|
   3   │ USER|MMI|5.18|Malignant neoplasm of lung|C0242379|[neop]|["Lung Cancer"-tx-1-"lung cancer"-noun-0]|TX|0/11|
   4   │ USER|MMI|5.18|Primary malignant neoplasm of lung|C1306460|[neop]|["Lung cancer"-tx-1-"lung cancer"-noun-0]|TX|0/1
       │ 1|
```

As you can see the output is prefaced with a log-line of my metamap installation. This line must be removed BEFORE running the mmi-parser.

In other words, we expect `metamap_results.txt` to contain:

```bash
USER|MMI|5.18|Carcinoma of lung|C0684249|[neop]|["LUNG CANCER"-tx-1-"lung cancer"-noun-0]|TX|0/11|
USER|MMI|5.18|Malignant neoplasm of lung|C0242379|[neop]|["Lung Cancer"-tx-1-"lung cancer"-noun-0]|TX|0/11|
USER|MMI|5.18|Primary malignant neoplasm of lung|C1306460|[neop]|["Lung cancer"-tx-1-"lung cancer"-noun-0]|TX|0/11|
```

You could try to hack your way around piping the output of MetaMap to this tool but it is beyond the scope for our use case.

### mmi-parser (CLI)

Once you have some MetaMap output, you can parse it into jsonlines simply by specifying the folder in which your output lives. The `mmi-parser` will go through each line in each `.txt` file in the specified directory and parse it into jsonlines.

For example, in this repo there is a provided [`data/sample.txt`](data/sample.txt) which contains the sample MMI output from the explanatory document linked at the top of this file.

You can run `mmi-parser` on this file by simply running:

```bash
cargo run --example parse
```

This runs [`examples/parse.rs`](examples/parse.rs) which passes `data` as the target directory to the `mmi-parser` tool. It is effectively the same as running `mmi-parser data` in your terminal. You can then see the jsonlines output in your [`data/sample_parsed.jsonl`](data/sample_parsed.jsonl) which is also provided in this repo.

### mmi-parser (API)

If you wish to use the mmi-parser crate in your application the easiest and most convenient method is to create an `MmiOutput` type by passing a string reference (most likely a single line of fielded MMI data from a file) to `parse_mmi()`.

Full API documentation can be found on [docs.rs](https://docs.rs/mmi-parser/latest/mmi_parser/).

## Support

If you encounter any issues or need support please either contact @nanthony007 or [open an issue](https://github.com/UK-IPOP/mmi-parser-rs/issues/new).

## Contributing

Pull requests are welcome. For major changes, please open an issue first to discuss what you would like to change.

Please make sure to update tests as appropriate.

See [CONTRIBUTING.md](CONTRIBUTING.md) for more details. 😃

## MIT License

[LICENSE](LICENSE)
