# Polylang

This is the source code for the `Polylang` documentation site at [polylang-site](https://polylang.dev/).

## Starting local Prover

For proof verification, the playground needs to communicate with the `prover` server.

First, run the `prover` as a local instance (the `server` package) from the project root (`polylang`):

```bash
$ cargo run -p server
```

Navigate to the site root (`polylang/site`):

```bash
$ cd site
```

All subsequent commands wil be executed inside this directory.

Copy the `.env.local.example` file to `.env.local` (this is a one-time operation).

```bash
$ cp .env.local.example .env.local
```

Confirm the contents:

```bash
$ more .env.local
NEXT_PUBLIC_PROVER_URL=http://localhost:8080/prove
```

The `NEXT_PUBLIC_PROVER_URL` is the URL of the `prover` instance. If you're hosting it on a different machine (or port), adjust values accordingly.

## Build and Run

Install the dependencies:

```bash
  $ yarn install
```

Build the project (optimized build):

```bash
  $ yarn build
```

Start the server:

```bash
  $ yarn start
```

### Development

```bash
  $ yarn dev
```

This will spin up the local development server on [localhost:3000](localhost:3000).

## Contribution

`Polylang` is Free and Open Source Software. We welcome bug reports and patches from everyone.

For more information on contribution tips and guidelines, please see the [Contributing](https://github.com/polybase/polylang/blob/main/CONTRIBUTING.md) page.

Note: If you wish to report issues with the `Polylang` documentation (this site), feel free to [open an issue](https://github.com/polybase/polylang/issues).

## LICENSE

This repository is licensed under the [MIT License](LICENSE.md).