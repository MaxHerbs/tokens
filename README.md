[![CI](https://github.com/maxherbs/tokens/actions/workflows/ci.yml/badge.svg)](https://github.com/maxherbs/tokens/actions/workflows/_code.yml)
[![GitHub last commit](https://img.shields.io/github/last-commit/maxherbs/tokens.svg)](https://github.com/maxherbs/tokens/commits)
[![Coverage](https://codecov.io/gh/maxherbs/tokens/branch/main/graph/badge.svg)](https://codecov.io/gh/maxherbs/tokens)
[![License](https://img.shields.io/badge/License-Apache%202.0-blue.svg)](https://www.apache.org/licenses/LICENSE-2.0)

# Tokens

Simple CLI tool to manage OAuth clients and retrieve JWT. Store clients and their refresh tokens for quick access.

# Installing the Tool

Run the following to install the latest version. Then add the executable to PATH.

```bash
curl -L https://github.com/MaxHerbs/tokens/releases/latest/download/tokens.zip -o tokens.zip
unzip tokens.zip
chmod +x tokens
```

# Usage

## Adding Clients

To save a client, run the `add` command. This will store the client for use later.

```bash
tokens add --nickname <NAME> --auth-url https://<DOMAIN>/realms/master  --client-id <CLIENT-ID>
```

## Getting Tokens

To get a token, run `get <NICKNAME>`. If the client has a valid refresh token stored, the token will be used. If not, it will prompt for your username and password, and store the token.

```bash
tokens get <NICKNAME>
```

## View Saved Clients

Tokens has a `list` option to view saved clients.

```bash
tokens list
```

## Remove a Stored Client

Delete an existing client with the `delete` command.

```bash
tokens delete <NICKNAME>
```
