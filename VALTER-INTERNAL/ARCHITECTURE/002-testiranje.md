### ADR-002: Standardi za Testiranje Rust Koda (V1)


### Kako Funkcionira Testiranje u Rustu: Objašnjenje

Rust ima jedinstven i moćan pristup testiranju koji je ugrađen direktno u jezik i alatni lanac (Cargo), što eliminira potrebu za vanjskim frameworkovima poput Jesta (JavaScript) ili JUnita (Java) za osnovne potrebe.

#### Ključni Koncepti:

1.  **Testovi su Funkcije:** U Rustu, test je jednostavno funkcija koja je označena s `#[test]` atributom. Test runner prolazi ako funkcija završi bez panike, a pada ako dođe do panike.
2.  **Asercije (Assertions):** Panika se najčešće izaziva putem makroa za aserciju:
    *   `assert!(izraz)`: Pada ako je `izraz` `false`.
    *   `assert_eq!(lijevo, desno)`: Pada ako `lijevo` nije jednako `desno`. Prikazuje obje vrijednosti za lakše debugiranje.
    *   `assert_ne!(lijevo, desno)`: Pada ako su vrijednosti jednake.
3.  **Tri Vrste Testova:** Rust formalno razlikuje tri načina pisanja testova, svaki sa svojom svrhom i lokacijom.

    *   **a) Unit Testovi (Jedinični testovi):**
        *   **Svrha:** Testiranje malih, izoliranih dijelova koda (jedne funkcije, jedne metode) unutar jednog modula. Ključno je da **mogu pristupiti privatnim funkcijama i strukturama** unutar istog modula.
        *   **Lokacija:** Pišu se unutar `src/` direktorija, **u istom fileu kao i kod koji se testira**. Stavljaju se u poseban pod-modul nazvan `tests` i označen s `#[cfg(test)]`.
        *   **`#[cfg(test)]`:** Ovo je direktiva za kompajler koja mu govori: "Ovaj kod kompajliraj i uključi u binarnu datoteku **samo** kada se pokreće `cargo test`". U produkcijskom buildu (`cargo build --release`), cijeli testni modul se ignorira, tako da ne zauzima prostor i ne usporava kod.

    *   **b) Integracijski Testovi:**
        *   **Svrha:** Testiranje javnog API-ja vašeg "cratea" (biblioteke) kao cjeline. Ovi testovi simuliraju kako bi vanjski korisnik (ili drugi dio vašeg programa) koristio vašu biblioteku. Oni **mogu pristupiti samo javnim (`pub`) funkcijama i strukturama**.
        *   **Lokacija:** Pišu se u posebnom direktoriju `tests/` na istoj razini kao i `src/`. Svaki `.rs` file unutar `tests/` direktorija se tretira kao potpuno odvojeni, mali program (crate) koji ovisi o vašem glavnom crateu.
        *   **Kako rade:** Cargo automatski pronalazi ove fileove, kompajlira svaki od njih i povezuje ih s vašom bibliotekom, a zatim pokreće `#[test]` funkcije unutar njih.

    *   **c) Dokumentacijski Testovi (Doc Tests):**
        *   **Svrha:** Osigurati da su primjeri koda u vašoj dokumentaciji (komentarima) točni i ažurni. Ovo je fantastična značajka za održavanje kvalitetne dokumentacije.
        *   **Lokacija:** Pišu se direktno unutar `///` ili `//!` komentara kao blokovi koda ograđeni s ` ``` `.
        *   **Kako rade:** `cargo test` automatski pronalazi ove primjere, kompajlira ih kao male testove i izvršava. Ako se primjer više ne može kompajlirati zbog promjene API-ja, test će pasti, tjerajući vas da ažurirate dokumentaciju.

---


Sada, formalizirajmo sve ovo u ADR dokument koji možete dodati u `VALTER-INTERNAL/ARCHITECTURE/`.

[FILE_CONTENT_START ID="NEW_FILE" PATH="VALTER-INTERNAL/ARCHITECTURE/002-standardi-za-testiranje.md"]
### **ADR-002: Standardi za Testiranje Rust Koda**

**Status:** Potvrđeno
**Datum:** 2026-01-07
**Arhitekt:** Gemini

---

## 1. Kontekst

S rastom kompleksnosti VALTER projekta, neophodno je uspostaviti konzistentne, robusne i automatizirane standarde za testiranje. Cilj je osigurati visoku kvalitetu koda, spriječiti regresije (ponovno pojavljivanje starih bugova) i olakšati sigurno refaktoriranje koda. Ovaj dokument definira obvezujuća pravila i najbolje prakse za pisanje testova za sve Rust crateove unutar VALTER radnog prostora.

## 2. Odluka

Usvajamo slojeviti pristup testiranju koji se oslanja na ugrađeni Rust testni framework, uz dodatak specifičnih biblioteka za napredne scenarije poput mockinga i rada s datotečnim sustavom.

### 2.1. Filozofija Testiranja

1.  **Testovi su Prvorazredni Građani Koda:** Testovi nisu opcionalni. Svaka nova funkcionalnost ili ispravak buga mora biti popraćen odgovarajućim testovima.
2.  **Visoka Pokrivenost Javnog API-ja:** Svi `pub` (javni) itemi (funkcije, metode) moraju biti pokriveni barem jednim integracijskim testom.
3.  **CI je Čuvar Kvalitete:** Svi testovi se automatski izvršavaju u CI pipeline-u. Spajanje (merge) koda nije dozvoljeno ako ijedan test pada.

### 2.2. Korišteni Alati

*   **Core Framework:** Ugrađeni Rust test runner (`cargo test`).
*   **Asercije:** Standardni makroi: `assert!`, `assert_eq!`, `assert_ne!`.
*   **Rad s Filesystemom:** `tempfile` crate za kreiranje privremenih, izoliranih direktorija i datoteka, čime se osigurava da testovi ne ostavljaju smeće i ne ovise o vanjskom stanju.
*   **Mocking:** `mockall` crate za kreiranje mock objekata. Ovo je ključno za unit testiranje koda koji ovisi o vanjskim sustavima (npr. bazi podataka, mrežnim API-jima).

### 2.3. Struktura i Tipovi Testova

#### **a) Unit Testovi**
*   **Svrha:** Testiranje interne logike unutar jednog modula. Mogu pristupati privatnim funkcijama.
*   **Lokacija:** Unutar `src/`, u istom fileu kao i kod koji se testira, unutar `#[cfg(test)] mod tests { ... }`.
*   **Primjer (`core/src/aggregator.rs`):**
    ```rust
    // ... kod modula ...

    #[cfg(test)]
    mod tests {
        use super::*; // Uvozi sve iz roditeljskog modula (uključujući privatne iteme)

        #[test]
        fn internal_logic_test() {
            // ... test kod ...
            assert!(true);
        }
    }
    ```

#### **b) Integracijski Testovi**
*   **Svrha:** Testiranje javnog API-ja cratea iz perspektive vanjskog korisnika. Pristupaju samo `pub` itemima.
*   **Lokacija:** U `tests/` direktoriju unutar cratea (npr. `core/tests/`).
*   **Primjer (`core/tests/api_test.rs`):**
    ```rust
    // Uvozi naš crate kao da je vanjska ovisnost
    use valter_core::config::Config;

    #[test]
    fn public_api_function_test() {
        // ... kod koji poziva valter_core::neka_javna_funkcija() ...
        let result = Config::load("...`);
        assert!(result.is_ok());
    }
    ```

#### **c) Dokumentacijski Testovi**
*   **Svrha:** Osiguravanje točnosti primjera koda u dokumentaciji.
*   **Lokacija:** Unutar `///` komentara iznad javnih funkcija.
*   **Primjer:**
    ```rust
    /// Učitava konfiguraciju s putanje.
    ///
    /// # Primjer
    /// ```
    /// // Ovaj kod će se izvršiti kao test!
    /// // use valter_core::config::Config;
    /// // let config = Config::load("valter.config.example");
    /// // assert!(config.is_ok());
    /// ```
    pub fn load(path: &str) -> Result<Self> {
        // ...
    }
    ```

### 2.4. Praktične Smjernice za Pisanje Testova

#### **Testiranje Funkcija koje Vraćaju `Result`**
Testne funkcije mogu vraćati `Result`. Ovo omogućava korištenje `?` operatora za čišći kod.

```rust
#[test]
fn test_something_that_can_fail() -> Result<(), Box<dyn std::error::Error>> {
    let dir = tempfile::tempdir()?; // '?' se može koristiti
    let config = Config::load(dir.path().to_str().unwrap())?;
    assert_eq!(config.global.port, 8000);
    Ok(())
}
```

#### **Testiranje Panike**
Ako funkcija *treba* paničariti pod određenim uvjetima, koristite `#[should_panic]`.

```rust
#[test]
#[should_panic(expected = "Konfiguracija mora imati barem jedan CLOUD")]
fn test_panics_on_empty_clouds() {
    // ... pripremi neispravan config ...
    Config::load("neispravan.config").unwrap();
}
```

#### **Testiranje Filesystema i Baze Podataka**
Uvijek koristite `tempfile` za kreiranje privremenih datoteka/direktorija i `rusqlite::Connection::open_in_memory()` za in-memory bazu podataka. **Nikada ne izvršavajte testove na razvojnoj ili produkcijskoj bazi podataka.**

#### **Mocking Eksternih Servisa**
Za testiranje modula koji npr. komunicira s Gemini API-jem, koristite `mockall` da simulirate API odgovor bez stvarnog mrežnog poziva.

```rust
// Primjer koncepta (zahtijeva `mockall` setup)
//
// let mut mock_api_client = MockApiClient::new();
// mock_api_client.expect_ask_oracle()
//                .with(eq("pitanje"))
//                .times(1)
//                .returning(|| Ok("odgovor".to_string()));
//
// let result = service_that_uses_client.do_work(&mock_api_client, "pitanje");
// assert_eq!(result, "odgovor");
```

### 2.5. Izvršavanje i CI

*   **Lokalno:** `cargo test --workspace` pokreće sve testove.
*   **Lokalno s ispisom:** `cargo test -- --nocapture` pokreće testove i prikazuje `println!` ispise.
*   **CI Pipeline:** `.github/workflows/ci.yml` mora sadržavati korak `run: cargo test --workspace`.

## 3. Posljedice

*   **Prednosti:**
    *   Značajno povećana pouzdanost i stabilnost koda.
    *   Smanjen broj bugova koji stižu do produkcije.
    *   Lakše i sigurnije refaktoriranje.
    *   Testovi služe kao izvršna dokumentacija o tome kako kod treba raditi.
    *   Novi članovi tima imaju jasne smjernice za pisanje kvalitetnog koda.
*   **Nedostaci:**
    *   Povećava se inicijalno vrijeme potrebno za razvoj novih funkcionalnosti.
    *   Loše napisani testovi mogu biti krhki (brittle) i pucati na bezazlene promjene, usporavajući razvoj.

