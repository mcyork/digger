
digger: a command line tool to query DNS against an instance of the dns-checker API.

check-dns provides a web interface and API for DNS. It may not be rocket science, but it has a use case for helping those inside larger enterprises to navigate and see split DNS responses.

You can find the project on GitHub: [check-dns](https://github.com/mcyork/check-dns)

Why digger: I wanted to write a CLI program to challenge myself and cover another base. The first base being the check-dns Flask app. Here, I explored Rust and GitHub build YAML. Since I cannot compile on three different platforms, I used GitHub Actions to compile three binary versions into a "release".

```shell
% chmod +x digger-macos 
% ./digger-macos 
error: The following required arguments were not provided:
    <DOMAIN>
    <TYPE>

USAGE:
    digger-macos <DOMAIN> <TYPE>

For more information, try --help
% 
% ./digger-macos --help
digger 1.0
A Rust CLI tool to perform DNS lookups using a specified API

USAGE:
    digger-macos [OPTIONS] [ARGS]

ARGS:
    <DOMAIN>    The domain to look up
    <TYPE>      The DNS record type

OPTIONS:
        --config         Show the current configuration
    -h, --help           Print help information
        --setup <URL>    Set the API URL
    -V, --version        Print version information
% 
```
