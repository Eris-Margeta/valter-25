### **ADR-003: Rješavanje Ovisnosti o Build Artefaktima pri Testiranju**

**Status:** Potvrđeno
**Datum:** 2026-01-07
**Arhitekt:** Eris

---

## 1. Kontekst

Tijekom implementacije sustava za testiranje, naišli smo na kritičnu grešku pri pokretanju `cargo test`. Kompilacija `core` cratea nije uspijevala s greškom `folder '/app/dist' does not exist`.

Problem proizlazi iz `core/src/api.rs` datoteke, koja koristi `rust-embed` crate za ugradnju (embed) frontend web aplikacije direktno u Rust binarnu datoteku. Makro `#[derive(RustEmbed)]` s atributom `#[folder = "../app/dist"]` izvršava se tijekom kompajliranja i zahtijeva da navedeni direktorij fizički postoji na disku u tom trenutku.

Standardna naredba `cargo test` pokreće samo kompajliranje i testiranje Rust koda. Ona ne pokreće vanjske procese, poput `pnpm build`, koji su potrebni za generiranje `app/dist` direktorija. To dovodi do neuspjeha kompajliranja u testnom okruženju i blokira cijeli CI/CD pipeline.

## 2. Razmatrane Opcije

### Opcija 1: Uvjetna Kompilacija (`#[cfg]`) s Dvije Strukture

Prvi pokušaj rješenja bio je definirati dvije odvojene `Assets` strukture: jednu za produkciju koja koristi `rust-embed` i jednu lažnu (mock) za testno okruženje.

```rust
#[cfg(not(test))]
#[derive(RustEmbed)]
#[folder = "../app/dist"]
struct Assets;

#[cfg(test)]
struct Assets; // Lažna struktura za test
```

*   **Problem:** Ovaj pristup nije uspio. `derive` makro se izvršava u ranoj fazi kompajliranja i čini se da provjerava postojanje foldera čak i za neaktivni `cfg` blok. Kompilacija je i dalje pucala.

### Opcija 2: Uvjetni Atributi (`#[cfg_attr]`)

Drugi pokušaj bio je korištenje jedne `Assets` strukture i dinamičko mijenjanje `folder` atributa ovisno o tome da li se kompajlira za testove.

```rust
#[derive(RustEmbed)]
#[cfg_attr(not(test), folder = "../app/dist")]
#[cfg_attr(test, folder = "tests/fixtures/")] // Prazan folder za testove
struct Assets;
```

*   **Problem:** I ovaj pristup je propao iz istog razloga. Čini se da `derive` makro validira sve potencijalne putanje definirane u `cfg_attr` atributima, a kako `../app/dist` nije postojao tijekom `cargo test`, kompilacija je opet bila neuspješna.

### Opcija 3: Korištenje Build Skripte (`build.rs`)

Ovaj pristup mijenja strategiju: umjesto da pokušavamo promijeniti Rust kod da se prilagodi nepostojećem direktoriju, osiguravamo da direktorij **uvijek postoji** prije nego što kompajler uopće dođe do `api.rs`.

*   **Logika:** Cargo je dizajniran da izvrši `build.rs` skriptu **prije** kompajliranja ostatka cratea. Možemo iskoristiti tu skriptu da provjerimo postoji li `app/dist` i, ako ne, da ga kreiramo.

## 3. Odluka

Usvajamo **Opciju 3**. Koristit ćemo `build.rs` skriptu unutar `core` cratea kako bismo programatski osigurali da `app/dist` direktorij postoji prije početka glavne faze kompajliranja.

### Obrazloženje

Ovo rješenje je superiorno iz nekoliko razloga:

1.  **Pouzdanost:** Garantirano rješava problem jer se izvršava u pravo vrijeme u build procesu.
2.  **Jednostavnost Koda:** Održava `api.rs` čistim i jednostavnim. Uklanja svu potrebu za kompleksnom `#[cfg]` i `#[cfg_attr]` logikom oko `Assets` strukture.
3.  **Ispravna Podjela Odgovornosti:** Logika koja se tiče pripreme build okruženja (kao što je osiguravanje postojanja direktorija) pripada u build skriptu, a ne u logiku aplikacije.
4.  **Niski Overhead:** `fs::create_dir_all` je idempotentna i vrlo brza operacija. Neće raditi ništa ako direktorij već postoji, tako da ne usporava regularni `pnpm build` tijek rada.

## 4. Detaljna Implementacija

#### A. Ažuriranje `core/build.rs`

Sljedeći kod se dodaje na vrh `core/build.rs` datoteke:

```rust
use std::env;
use std::fs;
use std::path::Path;

fn main() {
    // RJEŠENJE ZA `cargo test`
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let dist_path = Path::new(&manifest_dir).join("..").join("app").join("dist");

    if !dist_path.exists() {
        fs::create_dir_all(&dist_path)
            .expect("Failed to create a dummy dist directory for rust-embed");
    }

    // ... ostatak build.rs skripte ...
}
```

#### B. Pojednostavljenje `core/src/api.rs`

`Assets` struktura se vraća na svoju najjednostavniju formu:

```rust
use rust_embed::RustEmbed;

#[derive(RustEmbed)]
#[folder = "../app/dist"]
struct Assets;
```
Sva logika `static_handler`-a je također prilagođena da radi s `rust-embed` v8+ API-jem.

## 5. Posljedice

*   **Pozitivno:** `cargo test` se sada izvršava uspješno bez grešaka. CI/CD pipeline je stabilan. Logika u `api.rs` je čišća.
*   **Negativno:** Postoji minimalni, zanemarivi overhead pri svakom `cargo` pozivu jer `build.rs` skripta uvijek provjerava postojanje direktorija. Uvedena je "skrivena" ovisnost gdje `core` build skripta modificira stanje izvan svog direktorija, što čini ovaj ADR dokument ključnim za razumijevanje te veze.

