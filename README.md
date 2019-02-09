Kin Backup
==========

Secure, simple backups for your [next of kin](https://en.wikipedia.org/wiki/Next_of_kin).

This project serves two main goals:

1. Make it easier to create and maintain secure backups of sensitive information that my non-technical close relatives can restore after I am hit by the proverbial Bus.
2. Help me learn Rust.

Overview
--------

Kin is a simple system that creates backup packages for your relatives to keep locked away in fire safes, safe deposit boxes, etc. It should definitely _not_ be used as an online backup, is definitely _not_ meant to be given to untrustworthy people, and will definitely _not_ protect you from the NSA.

Each backup package contains an unencrypted zip archive of public documents, for things like [last will and testament](https://en.wikipedia.org/wiki/Will_and_testament) documents or [living trust](https://en.wikipedia.org/wiki/Trust_law) documents. It also contains an encrypted archive for things like password manager backups, two-factor authentication recovery codes, sensitive personal information, etc. Encryption is implemented using [libsodium](https://download.libsodium.org/doc/).

Of course giving any one person such sensitive "keys to the kingdom" is arguably a bad idea. Which is why Kin prevents any one individual from being able to decrypt the backup that they have access to. Kin is designed so that you give backups to at least two different people. Each person cannot decrypt their own backup, but they can decrypt a backup that someone else holds. This isn't just to protect you from one family member going rogue, but also protects you when your trusted backup-holders experience burglary or otherwise lose track of your backups.

Kin also helps generate a clear, concise, non-technical readme for your relatives. You can customize the readme however you want. Each backup comes with a simple decryption program that runs on Linux, and in version 1.0 it will provide an executable for Windows and Mac OS as well.

Build Requirements
------------------

Kin Backup is created using Rust, so you need a Rust development environment. Since we're using libsodium, to build this project you'll also need a C compiler (`cc`, `clang`, etc) and `libssl-dev` (assuming you're on Ubuntu).
