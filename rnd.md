# RND Command

## Description

Manage research and development (RND) projects on Google Cloud

## Usage

```bash
des rnd <subcommand> [args...]
```

## Subcommands

| Subcommand  | Description                                        |
| ----------- | -------------------------------------------------- |
| preprocess  | Run preprocessing steps for the RND project        |
| run         | Run the RND application locally                    |
| info        | Get information about the deployed RND application |
| deploy      | Deploy the RND application to Google Cloud Run     |
| reconfigure | Update the Google Cloud Run configuration          |
| destroy     | Remove the deployed RND application                |
| preload     | Load data for the RND project                      |
| upload      | Upload data to the RND project's storage bucket    |

## Details

This command provides utilities for managing research and development (RND) projects on Google Cloud. It handles various aspects of the RND workflow, including data preprocessing, local execution, deployment, and management of Google Cloud resources.

## Examples

```bash
des rnd preprocess
des rnd run
des rnd deploy
des rnd info
des rnd destroy
```
