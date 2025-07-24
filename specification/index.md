# Overview

Create a rust library and documentation for acquiring markdown from URLs.

This is different from 'just downloading' because we are going to smart handle URLs.

## General

Use your best judgement on creating markdown that is well organized.

Remember we will be using this markdown to feed into LLM and prompts.

Include YAML front matter in all markdown generated that includes:

- the source URL
- what exporter or post processing was used
- the date downloaded

## Use Cases

### Download

As a library user, I want a unified API with an Url in and Markdown out.

The Markdown type should bew a newtype of string with all the fixings.

### HTML Pages

As a library user, when I download a generic URL, convert the results with html2md.

### Google Docs

As a library user, when I download a google doc by URL, append the correct export format to get a markdown file.

### Office 365

As a library user, when I download an Office 365 doc by URL, apppend the correct export format to get a markdown file.

### Github Issues

As a library user, when I download a GitHub Issue, use the github REST api to fetch the issue along with associated comments
and render that to markdown.
