[![Contributors][contributors-shield]][contributors-url]
[![Forks][forks-shield]][forks-url]
[![Stargazers][stars-shield]][stars-url]
[![Issues][issues-shield]][issues-url]
[![Apache 2.0 License][license-shield]][license-url]




<h1 align="center">Listmonk Sync</h1>
<p align="center">
  A tool to automatically sync <a href="https://squareup.com/">Square</a> customers to <a href="https://listmonk.app/">listmonk</a>
  <br />
  <br />
  <a href="https://github.com/Insprill/listmonk-sync/issues">Report Bugs</a>
  ·
  <a href="https://github.com/Insprill/listmonk-sync/issues">Request Features</a>
</p>




<!-- TABLE OF CONTENTS -->
<details>
  <summary><h2 style="display: inline-block">Table of Contents</h2></summary>
  <ol>
    <li><a href="#deployment">Deployment</a></li>
    <li><a href="#compiling">Compiling</a></li>
    <li><a href="#contributing">Contributing</a></li>
    <li><a href="#license">License</a></li>
  </ol>
</details>




<!-- DEPLOYMENT -->

## Deployment

### Requirements

To host listmonk-sync, you'll need an API key from [Square](https://developer.squareup.com/) and your listmonk credentials.

### Docker

The easiest way to host listmonk-sync is via Docker.  
First, clone this repo and cd into the directory.
```
git clone https://github.com/Insprill/listmonk-sync
cd listmonk-sync
```
Next, open the `docker-compose.yml`, find `SQUARE_API_TOKEN: "token"` and replace `token` with the token with your Square access token.
Then, set `LISTMONK_USER` and `LISTMONK_PASSWORD` the same way.

To finish the setup, open the `config.json` file and fill in the provided parameters. For more information, checkout the [configuration section](#configuration).

Finally, you can start the container.
```
docker compose up -d
```




<!-- Configuration -->

## Configuration

|Key|Description|Default|
|-|-|-|
|`run_every`|How often sync's should run. A sync will run when the program is started, then every n seconds.|`3600`|
|`listmonk_domain`|The domain of your listmonk instance. Shouldn't include the scheme or a subdirectory.|`example.com`|
|`listmonk_list_ids`|The IDs of the lists to add the imported customers to. You can find the ID at the top of the edit panel.|`1,2,3`|
|`listmonk_confirmation`|Whether the imported customers should be marked as confirmed.|`true`|
|`listmonk_overwrite`|Whether subscribers already in listmonk should be overwritten.|`false`|




<!-- Compiling -->

## Compiling

To compile BuildTools Assistant, you'll need [Rust](https://www.rust-lang.org/tools/install).  
Clone this repo, then run `cargo build --release` from your terminal.  
You can find the compiled program in the `target/release` directory.  




<!-- CONTRIBUTING -->

## Contributing

Contributions are what make the open source community such an amazing place to learn, inspire, and create.  
While this project was made for my own needs, if you find it useful and would like to contribute, that'd be greatly appreciated!  
If you're new to contributing to open-source projects, you can follow [this](https://docs.github.com/en/get-started/quickstart/contributing-to-projects) guide.




<!-- LICENSE -->

## License

Distributed under the Apache 2.0 License. See [`LICENSE`][license-url] for more information.




<!-- MARKDOWN LINKS & IMAGES -->
<!-- https://www.markdownguide.org/basic-syntax/#reference-style-links -->

[contributors-shield]: https://img.shields.io/github/contributors/Insprill/listmonk-sync.svg?style=for-the-badge
[contributors-url]: https://github.com/Insprill/listmonk-sync/graphs/contributors
[forks-shield]: https://img.shields.io/github/forks/Insprill/listmonk-sync.svg?style=for-the-badge
[forks-url]: https://github.com/Insprill/listmonk-sync/network/members
[stars-shield]: https://img.shields.io/github/stars/Insprill/listmonk-sync.svg?style=for-the-badge
[stars-url]: https://github.com/Insprill/listmonk-sync/stargazers
[issues-shield]: https://img.shields.io/github/issues/Insprill/listmonk-sync.svg?style=for-the-badge
[issues-url]: https://github.com/Insprill/listmonk-sync/issues
[license-shield]: https://img.shields.io/github/license/Insprill/listmonk-sync.svg?style=for-the-badge
[license-url]: https://github.com/Insprill/listmonk-sync/blob/master/LICENSE
