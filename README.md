# XabelFish
*ni**X** b**abelFish**

This is a live translator for some visual novel game players who use unix-like operating systems.
This software is hard-coded for japanese game. Change it if you want.

You need to change DeepL API Key with Settings menu.

## Requirements
- Tesseract
- Pipewire
- XDG Desktop Portal
    - You may need to install xdg-desktop-portal and one or more backend. Consult [ArchWiki documentation](https://flatpak.github.io/xdg-desktop-portal/docs/doc-org.freedesktop.portal.ScreenCast.html) for more details

## How to build
1. Install rust
1. Install node.js
1. Install [pnpm](https://pnpm.io/)
1. Run `pnpm tauri build`

## TO-DO
- [ ] Support other OCR engines
  - [ ] EasyOCR
  - [ ] ppocr?
- [x] Add tessearact source lanaguge (currently hard-coded to `jpn`)
- [ ] Add translator option (target language, source language)
- [x] Add font settings
- [ ] Add support for other translating engines
  - [ ] LibreTranslate
- [ ] Add image-to-image translation

## License
XabelFish - Game translator for Unix-like operating systems
<br>
Copyright (C) 2025 Yeonjin Shin (a.k.a. LiteHell)

This program is free software: you can redistribute it and/or modify
it under the terms of the GNU General Public License as published by
the Free Software Foundation, either version 3 of the License, or
(at your option) any later version.

This program is distributed in the hope that it will be useful,
but WITHOUT ANY WARRANTY; without even the implied warranty of
MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
GNU General Public License for more details.

You should have received a copy of the GNU General Public License
along with this program.  If not, see <https://www.gnu.org/licenses/>.