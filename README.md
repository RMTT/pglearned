# pglearned

Replace components of postgresql by machine learning model.


## Requirements
+ postgresql: >= 18

## Usage

pglearned contains several components, the core is a [postgresql extension](extension). To use pglearned, you should Install this extension to your database first. You can visit [extension document](extension/README.md) to get more information.

There is a [cli tool](cli) can be used to manage datasets(sql queries) in your database.

There are also some frameworks for communicating with pglearned extension:
+ python: [document](frameworks/python)

## Development

enter dev env via nix:
```shell
nix develop .
```
