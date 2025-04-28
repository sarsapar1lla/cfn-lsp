# cfn-lsp

[![Build workflow](https://github.com/sarsapar1lla/cfn-lsp/actions/workflows/workflow.yaml/badge.svg)](https://github.com/sarsapar1lla/cfn-lsp/actions/workflows/workflow.yaml)

Language Server Protocol (LSP) implementation for AWS CloudFormation

## Installation

Install from source using `cargo`:

```bash
$ cargo install --git https://github.com/sarsapar1lla/cfn-lsp
```

## Usage

### Std IO

To communicate via StdIn/Out, use:

```bash
$ cfn-lsp stdio
```

### TCP

To communicate via TCP, use:

```bash
$ cfn-lsp socket --port 1234
```

> **NB**: The LSP client process id can be provided using the `--clientProcessId` flag (alias `--client-process-id`)

## Local Development

Build the project using `cargo`:

```bash
$ cargo build
```

Run all tests using:

```bash
$ cargo test
```
