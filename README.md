# smartrelease - Redirect to release assets dynamically

[Releases](https://docs.github.com/en/repositories/releasing-projects-on-github/about-releases) are an essential feature of github (or other platforms like [gitea](https://gitea.io)) and the assets you can attach are pretty useful if you want to pre-compile a binary for example.
But linking to these assets directly in your README can be pretty annoying if every of your release asset has a version number in it, for example `program-v1.0.0`, and with every release the version number changes, and you have to change the direct link to it in your README.
And this is where **smartrelease** enters the game.
It provides a simple yet powerful api endpoint which will redirect the user directly to the latest release asset you've specified in the api url. 

<p align="center">
  <a href="https://github.com/ByteDream/smartrelease/releases/latest">
    <img src="https://img.shields.io/github/v/release/ByteDream/smartrelease?style=flat-square" alt="Latest release">
  </a>
  <a href="https://github.com/ByteDream/smartrelease/blob/master/LICENSE">
    <img src="https://img.shields.io/github/license/ByteDream/smartrelease?style=flat-square" alt="License">
  </a>
  <a href="#">
    <img src="https://img.shields.io/github/languages/top/ByteDream/smartrelease?style=flat-square" alt="Top language">
  </a>
  <a href="https://discord.gg/gUWwekeNNg">
    <img src="https://img.shields.io/discord/915659846836162561?label=discord&style=flat-square" alt="Discord">
  </a>
</p>

## How it's working

<p align="center"><strong>Take me directly to the <a href="#examples">examples</a>.</strong></p>

The api works with a single link which will take the user to the asset of the latest release you've specified.
Its url scheme looks like the following:
<pre>
<code>https://example.com/<a href="#platform">:platform</a>/<a href="#owner">:owner</a>/<a href="#repository">:repository</a>/<a href="#pattern">:pattern</a></code>
</pre>

### platform

The **platform** is the hoster of your git repository.
The current supported platforms are:
- [GitHub](https://github.com)
- [Gitea](https://gitea.com)

Currently, only the official instances are supported by the api (github has no instance for self-hosting anyway) but support for self-hostet instances is planned.

### owner

The **owner** is owner of the repository (mostly you).

### repository

The **repository** is the name of the repository you want to set up the api for.

### pattern

**pattern** is the part where all the magic happens.
The pattern you specify here is used to redirect the user to the asset you want.
To archive this, wildcards as well as regex (see [here](#warnings) for possible dangers with regex) can be used.

---

The following takes mostly care of wildcards since the [official instance](#official-instance) and default configurations are both only supporting wildcards and not regex.

Wildcards are simply nothing other than text fragments / variables / ... (whatever you want to call it) which are getting replaced with their corresponding values from the release name / tag.

These wildcards are currently supported:
- `major`

  _The major version number_.
  It must always be a number.
  In `smartrelease-v1.21.5-rc4`, `1` is the major version number.

  **Note**: The first number occurrence will always be matched as `major`.
  At the moment I have no idea how I should avoid this and only match the first number if necessary but if you want create a new [issue](https://github.com/ByteDream/smartrelease/issues/new), and I will take further investigations how to solve this.
- `minor`
  
  _The minor version number_.
  It must always be a number. 
  In `smartrelease-v1.21.5-rc4`, `21` is the minor version number.
- `patch`

  _The patch version number_.
  It must always be a number.
  In `smartrelease-v1.21.5-rc4`, `5` is the patch version number.
- `pre`

  _The pre-release number_. It can be a mix of numbers and letters (without any special character between).
  In `smartrelease-v1.21.5-rc4`, `rc4` is the pre-release number.
- `tag`

  _The release tag_. In `https://github.com/ByteDream/smartrelease/releases/tag/v0.1.0`, `v0.1.0` is the tag.

`major`, `minor`, `patch` and `pre` are all version number specific wildcards.
Hence, they are matched descending.
This means `minor` is only matched if `major` is matched, `patch` is only matched if `minor` is matched and so on.

---

I clearly can't name all cases here where the pattern matches your asset name or not, so if you want to check and test which name is support and which not, I suggest you to visit this [website](https://regex101.com/r/gU5vbe/1) and type in your asset name in the big box.
If it gets highlighted somewhere it is supported, if not then not.
In case your asset name is not supported, but you want it to be supported, feel free to create a new [issue](https://github.com/ByteDream/smartrelease/issues/new) or join the [discord server](https://discord.gg/gUWwekeNNg) and submit your asset name, so I can take care and implement it.

## Examples

For the example the [official instance](#official-instance) is used as host.

Latest release for this repo.
The result looks like this: [Latest release](https://smartrelease.bytedream.org/github/ByteDream/smartrelease/smartrelease-v{major}.{minor}.{patch}_linux)
```
[Latest release](https://smartrelease.bytedream.org/github/ByteDream/smartrelease/smartrelease-v{major}.{minor}.{patch}_linux)
```

We can also use [shields.io](https://shields.io) to make it look more appealing to the user.
The result looks like this: [![Latest release](https://img.shields.io/github/v/release/ByteDream/smartrelease?style=flat-square)](https://smartrelease.bytedream.org/github/ByteDream/smartrelease/smartrelease-v{major}.{minor}.{patch}_linux)
```
[![Latest release](https://img.shields.io/github/v/release/ByteDream/smartrelease?style=flat-square)](https://smartrelease.bytedream.org/github/ByteDream/smartrelease/smartrelease-v{major}.{minor}.{patch}_linux)
```

And now with the official Gitea instance (Gitea is a great open-source based alternative to GitHub, if you didn't knew it already)
The result looks like this: [Now with gitea!](https://smartrelease.bytedream.org/gitea/gitea/tea/tea-{major}.{minor}.{patch}-linux-amd64)
```
[Now with gitea!](https://smartrelease.bytedream.org/gitea/gitea/tea/tea-{major}.{minor}.{patch}-linux-amd64)
```

## Hosting

## Official instance

The official instance is hosted on `https://smartrelease.bytedream.org`.
It has regex disabled and a maximal pattern length of 70 character.

So if you want, for example, using the official api for this repo, the following link will do it:
```
https://smartrelease.bytedream.org/github/ByteDream/smartrelease/smartrelease-v{major}.{minor}.{patch}_linux
```

Nevertheless, I recommend you to host your own instance if you have the capabilities to do so since I cannot guarantee that my server will have a 100% uptime (but I will do my best to keep it online).
I also recommend you to visit this repo from time to time to see if something will change / has already changed with the official instance.

## Self-hosting

_All following instructions are specified for linux, but at least [building](#build-it-from-source) should on every platform too_.

### Docker

**Make sure you have [docker](https://docker.com) installed**.

Clone the repo via `git clone` or download the [zipfile](https://github.com/ByteDream/crunchyroll-go/archive/refs/heads/master.zip) and extract it.
Open a shell, enter the directory and follow the following commands:
```shell
[~/smartrelease]$ docker build -t smartrelease .
[~/smartrelease]$ docker run -p 8080:8080 smartrelease
```

### Binary

Download the latest linux binary from [here](https://smartrelease.bytedream.org/github/ByteDream/smartrelease/smartrelease-v{major}.{minor}.{patch}_linux) (built with musl, so should work on libc and musl systems).
Now simply execute binary and the server is up and running:
```shell
[~]$ ./smartrelease-v<version>_linux
```

### Build it from source

**Make sure you have the latest stable version of [rust](https://www.rust-lang.org/) installed**.

Clone the repo via `git clone` or download the [zipfile](https://github.com/ByteDream/crunchyroll-go/archive/refs/heads/master.zip) and extract it.
Open a shell, enter the directory and follow the following commands:
```shell
[~/smartrelease]$ cargo build --release
[~/smartrelease]$ ./target/release/smartrelease
```

## Configuration

Every configuration can be made with environment variables or via an `.env` file. See [.example.env](.example.env) for an example configuration.

### `HOST`

The host address. I don't really know a case where this has to be changed but to have the choice is always better.
Default is `0.0.0.0`.

### `PORT`

The port to serve the server on.
Default is `8080`.

### `ENABLE_REGEX`

Enable or disable regex support in the pattern.
Default is `false`.

### `MAX_PATTER_LEN`

Limits the maximal length the pattern can have.
Default is `70`.

## Warnings

It is recommended to limit the pattern length with [`MAX_PATTER_LEN`](#max_patter_len) if [`ENABLE_REGEX`](#enable_regex) is enabled since a too long pattern which is too complex could lead to an, wanted or unwanted, [ReDoS](https://en.wikipedia.org/wiki/ReDoS) attack.

If you [host it yourself](#self-hosting) it is highly recommended taking one of the "big" dns servers like `1.1.1.1` or `8.8.8.8` as your dns resolver.
The [actix-web](https://actix.rs/) library which handles all the network stuff sometimes takes up too much time to resolve a dns address when the dns server is, for example, your local router.
And most of the time when this happens a 504 timeout error is thrown and the api is practically unusable.

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for more details.
