# Thunderbird MBOX to CSV Extractor (Rust)

## Overview

This project provides a robust, streaming-based Rust tool to extract email metadata from Mozilla Thunderbird mbox files into a single CSV file.

It is designed for large-scale datasets, including multi-gigabyte mail archives, and prioritizes reliability over strict RFC compliance.

The extractor processes all mbox files in a Thunderbird profile and outputs structured rows containing:

* Message identifier
* Source file path
* Subject
* Sender
* Recipients (To, Cc, Bcc)
* Recipient type

Each recipient is expanded into a separate row while preserving a shared message identifier.

---

## Key Features

Streaming processing
Memory usage remains constant regardless of dataset size.

Recursive traversal
Processes all mbox files within a Thunderbird profile, including nested directories.

Strict mbox detection
Only files without extension and with valid mbox structure are processed.

Fault-tolerant parsing
Handles malformed messages, invalid encodings, and broken headers.

Manual header extraction
Avoids reliance on fragile parsers, ensuring data is extracted even from corrupted emails.

---

## Output Structure

The generated CSV contains the following columns:

message_id
source_file
subject
from
to
cc
bcc
recipient_type

Notes:

* Each recipient generates a separate row
* message_id links rows belonging to the same email
* source_file allows traceability to the original mbox

---

## Installation

### Requirements

* Rust (stable toolchain)
* Cargo

Install Rust if needed:

https://www.rust-lang.org/tools/install

---

## Build

Clone the repository and compile in release mode:

```
cargo build --release
```

The executable will be located at:

```
target/release/mbox_to_csv
```

---

## Usage

Run the tool with:

```
./target/release/mbox_to_csv <thunderbird_profile_dir> <output_csv>
```

Example:

```
./target/release/mbox_to_csv ~/.thunderbird output.csv
```

The CSV will be written to the specified path. If a relative path is used, it will be created in the current working directory.

---

## How It Works

The tool reads each mbox file line by line using buffered I O. Messages are separated using the standard mbox delimiter starting with:

```
From 
```

For each message:

* The mbox separator line is removed
* Leading noise is stripped
* Only the header block is extracted
* Header folding is normalized
* Key fields are manually parsed

This approach ensures compatibility with real-world email data, including malformed or partially corrupted messages.

---

## Limitations

This tool prioritizes robustness over strict standards compliance.

* Email address parsing is simplified and not fully RFC 5322 compliant
* Encoded headers (MIME encoded words) are not decoded
* Some malformed messages may yield partial results
* CSV output can become very large with many recipients

---

## Performance Considerations

* I O throughput is the primary bottleneck, not CPU
* Suitable for processing gigabytes of data
* Output size may exceed input size due to row expansion

---

## When to Use

This tool is appropriate for:

* Email corpus analysis
* Legal or forensic data extraction
* Metadata indexing
* Preliminary dataset preparation

---

## Future Improvements

Potential extensions include:

* Full MIME decoding
* RFC compliant address parsing
* Message-ID extraction
* Parallel processing
* Parquet output for large-scale analytics

---

## License

GNU GPL v3

---

## Disclaimer

This tool processes email metadata only. It does not interpret message bodies or attachments. Data quality depends on the integrity of the original mbox files.
