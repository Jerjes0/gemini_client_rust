# Gemini Client Rust

Gemini Client Rust is a Rust library for interacting with Google's Gemini API. This library provides a simple way to send prompts to the API and receive responses, making it easy to integrate generative language capabilities into your Rust applications. At this point, it is only built for a single purpose: Receive a prompt and a list and find the three closer words from the list that represent the prompt. This can be modified for similar tasks, but is not yet generalized.

## Features

- Send prompts to Google's Gemini API
- Parse responses and extract related content
- Easy-to-use function for generating related words based on user input

## Installation

Add the following to your `Cargo.toml`:

```toml
[dependencies]
gemini_client_rust = "0.1.3"