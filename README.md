# sendmail

This is a CLI wrapper for the amazing [`lettre`](https://lettre.rs) crate. It allows you to easiliy send email without leaving the comfort of your shell.

## Configuration

`sendmail` expects a configuration file for each account in the TOML format, located in `$XDG_CONFIG_HOME/sendmail` (defaults to `.config/sendmail`). Here's an example configuration:

```toml
# .config/sendmail/school.toml

name = "Robin Boers"
email = "4410@schravenlant.nl"

[smtp]
hostname = "smtp.gmail.com"
port = 587
username = "4410@schravenlant.nl"
```

The password is passed via the CLI, because I'm not comfortable with having credentials in plain text on my computer.

## Usage

With the configuration from above:

```shell
sendmail school hello-world.md \
  --subject "Hello World!" \
  --to "hor@schravenlant.nl" \
  --to "you@example.com" \
  --password "$(pass mail/school)"
```
